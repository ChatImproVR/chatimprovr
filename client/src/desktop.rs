use crate::desktop_input::{DesktopInputHandler, WindowController};
use crate::{project_dirs, Client, LoginFile, LoginInfo, Opt};
use anyhow::{format_err, Result};
use cimvr_common::glam::Mat4;
use cimvr_common::ui::GuiTabId;
use cimvr_engine::interface::system::Stage;
use directories::ProjectDirs;
use eframe::egui;
use egui::mutex::Mutex;
use egui::{Color32, DragValue, Label, RichText, Ui};
use egui_dock::{NodeIndex, Style, Tree};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
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
    Game,
    Plugin(GuiTabId),
}

struct ChatimprovrEframeApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    cimvr_widget: Arc<Mutex<ChatimprovrWidget>>,
    dock_tree: Tree<TabType>,
    tab_viewer: TabViewer,
    //login_screen: LoginScreen,
}

impl ChatimprovrEframeApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Opt) -> Result<Self> {
        let gl = cc
            .gl
            .clone()
            .expect("You need to run eframe with the glow backend");

        let dock_tree = Tree::new(vec![TabType::Game]);

        let cimvr_widget = Arc::new(Mutex::new(ChatimprovrWidget::new(gl, args)?));

        let tab_viewer = TabViewer::new(cimvr_widget.clone());

        Ok(Self {
            //login_screen: LoginScreen::new(args.clone())?,
            cimvr_widget,
            dock_tree,
            tab_viewer,
        })
    }
}

impl eframe::App for ChatimprovrEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update game state
        self.cimvr_widget.lock().update();

        // Draw game
        egui::CentralPanel::default().show(ctx, |ui| {
            egui_dock::DockArea::new(&mut self.dock_tree)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut self.tab_viewer);

            /*
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                show_game_widget(ui, self.cimvr_widget.clone());
            });
            ui.label("Drag to rotate!");
            */
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if gl.is_some() {
            self.cimvr_widget.lock().destroy();
        }
    }
}

fn show_game_widget(ui: &mut egui::Ui, cimvr_widget: Arc<Mutex<ChatimprovrWidget>>) {
    let (rect, response) =
        ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

    let mut widge = cimvr_widget.lock();

    // We want to collect input...
    if response.hovered() {
        ui.input(|inp| widge.input.handle_egui_input(&inp, rect))
    }

    // Set window size to pixel size of the widget
    let pixel_size = ui.ctx().screen_rect().size() * ui.ctx().pixels_per_point();
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

    let callback = egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, _painter| {
            widge.lock().paint();
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

    fn paint(&mut self) {
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
            client
                .render_frame(Mat4::IDENTITY, 0)
                .expect("Frame render");
        }

        /*
        TODO: Travel requests
        // Check for travel requests
        let travel_request = client.as_mut().and_then(|c| c.travel_request());

        if let Some(client) = &mut client {
            // Post update stage
            client
                .engine()
                .dispatch(Stage::PostUpdate)
                .expect("Frame post-update");

            // Upload messages to server
            client.upload().expect("Message upload");
        }

        // Check for travel requests
        if let Some(travel_request) = travel_request {
            login_screen.login_file.last_login_address = travel_request.address;
            client = login_screen.login(&gl);
        }
        */
    }
}

struct TabViewer {
    cimvr_widget: Arc<Mutex<ChatimprovrWidget>>,
}

impl TabViewer {
    pub fn new(cimvr_widget: Arc<Mutex<ChatimprovrWidget>>) -> Self {
        Self { cimvr_widget }
    }
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = TabType;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            TabType::Game => show_game_widget(ui, self.cimvr_widget.clone()),
            _ => (),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            TabType::Game => "Game".into(),
            TabType::Plugin(id) => id.clone().into(),
        }
    }
}
