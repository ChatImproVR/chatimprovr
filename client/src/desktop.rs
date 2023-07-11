use crate::desktop_input::{DesktopInputHandler, WindowController};
use crate::{project_dirs, Client, LoginFile, LoginInfo, Opt};
use anyhow::{format_err, Result};
use cimvr_common::glam::Mat4;
use cimvr_common::ui::{GuiInputMessage, GuiOutputMessage, GuiTabId, PartialOutput, GuiConfigMessage};
use cimvr_engine::interface::system::Stage;
use directories::ProjectDirs;
use eframe::egui::{self, FullOutput, Pos2, Shape, Vec2, Mesh};
use egui::mutex::Mutex;
use egui::{Color32, DragValue, Label, RichText, Ui};
use egui_dock::{NodeIndex, Style, Tree};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn mainloop(mut args: Opt) -> Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(350.0, 380.0)),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        ..Default::default()
    };
    eframe::run_native(
        "ChatImproVR",
        options,
        Box::new(|cc| Box::new(ChatimprovrEframeApp::new(cc, args).unwrap())),
    )
    .map_err(|e| format_err!("{:#}", e))
}

enum TabType {
    Game(bool),
    Plugin(GuiTabId),
}

struct ChatimprovrEframeApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    cimvr_widget: Arc<Mutex<ChatimprovrWidget>>,
    dock_tree: Tree<TabType>,
    tabs: HashMap<GuiTabId, Option<PartialOutput>>,
    game_is_tab_fullscreen: bool,
    //login_screen: LoginScreen,
}

impl ChatimprovrEframeApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Opt) -> Result<Self> {
        let gl = cc
            .gl
            .clone()
            .expect("You need to run eframe with the glow backend");

        let dock_tree = Tree::new(vec![TabType::Game(false)]);

        let mut widge = ChatimprovrWidget::new(gl, args)?;

        // Subscribe to input messages
        widge
            .client
            .as_mut()
            .unwrap()
            .engine()
            .subscribe::<GuiOutputMessage>();

        widge
            .client
            .as_mut()
            .unwrap()
            .engine()
            .subscribe::<GuiConfigMessage>();

        let cimvr_widget = Arc::new(Mutex::new(widge));

        Ok(Self {
            //login_screen: LoginScreen::new(args.clone())?,
            game_is_tab_fullscreen: false,
            cimvr_widget,
            dock_tree,
            tabs: Default::default(),
        })
    }
}

impl eframe::App for ChatimprovrEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update game state
        let mut widge = self.cimvr_widget.lock();
        widge.update();

        // Process GUI input messages
        let client = widge.client.as_mut().unwrap();

        for msg in client.engine().inbox::<GuiOutputMessage>() {
            // Open new tab for it!
            if !self.tabs.contains_key(&msg.target) {
                self.dock_tree
                    .push_to_first_leaf(TabType::Plugin(msg.target.clone()));
            }

            self.tabs.insert(msg.target, msg.output);
        }

        // Process GUI config messages
        for msg in client.engine().inbox::<GuiConfigMessage>() {
            match msg {
                GuiConfigMessage::TabFullscreen(is_tab_fullscreen) => self.game_is_tab_fullscreen = is_tab_fullscreen,
            }
        }

        widge.post_update();

        // Unlock, avoiding deadlock
        drop(widge);

        // Draw game
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tab_viewer = TabViewer {
                cimvr_widget: self.cimvr_widget.clone(),
                last_frame: &self.tabs,
            };

            if self.game_is_tab_fullscreen {
                // Draw game only
                use egui_dock::TabViewer;
                tab_viewer.ui(ui, &mut TabType::Game(true));
            } else {
                // Draw docking
                egui_dock::DockArea::new(&mut self.dock_tree)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut tab_viewer);
            }
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if gl.is_some() {
            self.cimvr_widget.lock().destroy();
        }
    }
}

fn show_game_widget(ui: &mut egui::Ui, cimvr_widget: Arc<Mutex<ChatimprovrWidget>>, is_tab_fullscreen: bool) {
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

    let mut widge = cimvr_widget.lock();

    // We want to collect input...
    let ppp = ui.ctx().pixels_per_point();
    ui.input(|inp| widge.input.handle_egui_input(&inp, rect, response.hovered()));

    let widget_size_pixels = rect.size() * ppp;
    let screen_size = ui.ctx().screen_rect().size() * ppp;

    // Set window size to pixel size of the widget
    let pixel_size = if is_tab_fullscreen { screen_size } else { widget_size_pixels };
    widge
        .input
        .events
        .push(cimvr_common::desktop::InputEvent::Window(
                cimvr_common::desktop::WindowEvent::Resized {
                    width: pixel_size.x as _,
                    height: pixel_size.y as _,
                },
        ));

    // We're a game, renfer once per frame
    ui.ctx().request_repaint();

    // Clone locals so we can move them into the paint callback:
    let widge = cimvr_widget.clone();

    let fullscreen = is_tab_fullscreen.then(|| (screen_size.x as _, screen_size.y as _));
    let callback = egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, _painter| {
            widge.lock().paint(fullscreen);
        })),
    };
    ui.painter().add(callback);
}

struct ChatimprovrWidget {
    input: DesktopInputHandler,
    window_control: Option<WindowController>,
    client: Option<Client>,
}

impl ChatimprovrWidget {
    fn new(gl: Arc<glow::Context>, mut args: Opt) -> Result<Self> {
        let client = Client::new(gl, args.login_info()?)?;
        Ok(Self {
            input: DesktopInputHandler::new(),
            client: Some(client),
            window_control: None,
        })
    }

    fn update(&mut self) {
        if let Some(client) = &mut self.client {
            if self.window_control.is_none() {
                self.window_control = Some(WindowController::new(client.engine()));
            }

            // Download messages from server
            client.download().expect("Message download");

            // Send input history
            self.input.get_history(client.engine());
            let gamepad_state = client.gamepad.update();
            client.engine().send(gamepad_state);

            // Pre update stage
            client
                .engine()
                .dispatch(Stage::PreUpdate)
                .expect("Frame pre-update");

            // Update stage
            client
                .engine()
                .dispatch(Stage::Update)
                .expect("Frame udpate");

            client.prep_render().expect("Render prep");
        }
    }

    fn post_update(&mut self) {
        if let Some(client) = &mut self.client {
            // Post update stage
            client
                .engine()
                .dispatch(Stage::PostUpdate)
                .expect("Frame post-update");

            // Upload messages to server
            client.upload().expect("Message upload");
            /*
               TODO: Re-implement window control
               self.window_control
               .get_or_insert_with(|| WindowController::new(client.engine()))
               .update(client.engine(), glutin_ctx.window());

            // Collect UI input
            egui_glow.run(glutin_ctx.window(), |ctx| client.update_ui(ctx));
            */
        }
    }

    fn destroy(&self) {
        //todo!()
    }

    fn paint(&mut self, fullscreen: Option<(i32, i32)>) {
        /*
        // Login page
        if client.is_none() {
        // Attempt to login via command line arg
        if args.connect.is_some() {
        client = login_screen.login(&gl);
        // Don't loop
        args.connect = None;
        }

        // Otherwise, use the GUI to login
        egui_glow.run(glutin_ctx.window(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
        if login_screen.show(ui) {
        client = login_screen.login(&gl);
        }
        });
        });
        }
        */

        if let Some(client) = &mut self.client {
            // Render frame
            if let Some((width, height)) = fullscreen {
                client.render.force_fullscreen(width, height);
            }
            client
                .render_frame(Mat4::IDENTITY, 0)
                .expect("Frame render");
            }

        /*
           TODO: Travel requests
        // Check for travel requests
        let travel_request = client.as_mut().and_then(|c| c.travel_request());
        */


        /*
        // Check for travel requests
        if let Some(travel_request) = travel_request {
        login_screen.login_file.last_login_address = travel_request.address;
        client = login_screen.login(&gl);
        }
        */
    }
}

struct TabViewer<'a> {
    cimvr_widget: Arc<Mutex<ChatimprovrWidget>>,
    last_frame: &'a HashMap<GuiTabId, Option<PartialOutput>>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = TabType;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            TabType::Game(is_tab_fullscreen) => {
                egui::Frame::canvas(ui.style())
                    .show(ui, |ui| show_game_widget(ui, self.cimvr_widget.clone(), *is_tab_fullscreen));
                }
            TabType::Plugin(id) => {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                let raw_input = ui
                    .ctx()
                    .input(|input_state| convert_subwindow_input(input_state, rect));

                // Send input events to host
                self.cimvr_widget
                    .lock()
                    .client
                    .as_mut()
                    .unwrap()
                    .engine()
                    .send(GuiInputMessage {
                        target: id.clone(),
                        raw_input,
                    });

                // Draw existing state
                if let Some(Some(full_output)) = self.last_frame.get(id) {
                    for mesh in &full_output.shapes
                    {
                        let clip = mesh.clip;
                        let mut mesh: Mesh = mesh.clone().into();

                        let offset = rect.left_top().to_vec2();
                        mesh.translate(offset);

                        ui.set_clip_rect(clip.translate(offset));
                        ui.painter().add(Shape::Mesh(mesh));
                    }
                }
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            TabType::Game(_) => "Game".into(),
            TabType::Plugin(id) => id.clone().into(),
        }
    }
}

fn convert_subwindow_input(input_state: &egui::InputState, rect: egui::Rect) -> egui::RawInput {
    let mut raw = input_state.raw.clone();

    raw.screen_rect = Some(egui::Rect::from_min_size(Pos2::ZERO, rect.size()));

    for ev in &mut raw.events {
        match ev {
            egui::Event::PointerMoved(new_pos) => {
                *new_pos -= rect.left_top().to_vec2();
            }
            egui::Event::PointerButton { pos, .. } => {
                *pos -= rect.left_top().to_vec2();
            }
            _ => (),
        }
    }

    raw
}
