use crate::desktop_input::{DesktopInputHandler, WindowController};
use crate::{project_dirs, Client, LoginFile, LoginInfo, Opt};
use anyhow::{format_err, Result};
use cimvr_common::glam::Mat4;
use cimvr_engine::interface::system::Stage;
use directories::ProjectDirs;
use eframe::egui;
use egui::mutex::Mutex;
use egui::{Color32, DragValue, Label, RichText, Ui};
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
        ..Default::default()
    };
    eframe::run_native(
        "ChatImproVR",
        options,
        Box::new(|cc| Box::new(ChatimprovrEframeApp::new(cc, args).unwrap())),
    )
    .map_err(|e| format_err!("{:#}", e))
}

struct ChatimprovrEframeApp {
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    cimvr_widget: Arc<Mutex<ChatimprovrWidget>>,
    login_screen: LoginScreen,
}

impl ChatimprovrEframeApp {
    fn new(cc: &eframe::CreationContext<'_>, args: Opt) -> Result<Self> {
        let gl = cc
            .gl
            .clone()
            .expect("You need to run eframe with the glow backend");
        Ok(Self {
            login_screen: LoginScreen::new(args.clone())?,
            cimvr_widget: Arc::new(Mutex::new(ChatimprovrWidget::new(gl, args)?)),
        })
    }
}

impl eframe::App for ChatimprovrEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.cimvr_widget.lock().destroy();
        }
    }
}

impl ChatimprovrEframeApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // We're a game, renfer once per frame
        ui.ctx().request_repaint();

        // Clone locals so we can move them into the paint callback:
        let widge = self.cimvr_widget.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                widge.lock().paint();
            })),
        };
        ui.painter().add(callback);
    }
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

/*
   pub fn old_mainloop(mut args: Opt) -> Result<()> {
// Set up window
let event_loop = glutin::event_loop::EventLoop::new();
let window_builder = glutin::window::WindowBuilder::new().with_title("ChatImproVR");

// Set up OpenGL
let glutin_ctx = unsafe {
glutin::ContextBuilder::new()
.with_vsync(true)
.build_windowed(window_builder, &event_loop)?
.make_current()
.unwrap()
};

let gl = unsafe {
gl::Context::from_loader_function(|s| glutin_ctx.get_proc_address(s) as *const _)
};
let gl = Arc::new(gl);

// Set up egui
let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

// Set up desktop input
let mut input = DesktopInputHandler::new();
let mut window_control = None;

// Setup client code
let mut client: Option<Client> = None;
let mut login_screen = LoginScreen::new(args.clone())?;

// Run event loop
event_loop.run(move |event, _, control_flow| {
 *control_flow = ControlFlow::Poll;
 match event {
 Event::MainEventsCleared => {
 glutin_ctx.window().request_redraw();
 }
 Event::RedrawRequested(_) =>
 Event::WindowEvent { ref event, .. } => {
 if !egui_glow.on_event(&event) {
 input.handle_winit_event(event);
 }

 match event {
 WindowEvent::Resized(ph) => {
 if let Some(client) = &mut client {
 client.set_resolution(ph.width, ph.height);
 }
 glutin_ctx.resize(*ph);
 }
 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
 _ => (),
 }
 }
 Event::LoopDestroyed => {
 egui_glow.destroy();
 }
 _ => (),
 }
 })
 }
 */

#[derive(Default)]
struct LoginScreen {
    login_file: LoginFile,
    err_text: String,
}

impl LoginScreen {
    pub fn new(args: Opt) -> Result<Self> {
        let mut login_file = LoginFile::load()?;
        if let Some(addr) = args.connect {
            login_file.last_login_address = addr;
        }
        if let Some(user) = args.username {
            login_file.username = user;
        }

        Ok(Self {
            login_file,
            err_text: "".into(),
        })
    }

    /// Takes gl as an argument in order to create client instance (nothing else!)
    pub fn login(&mut self, gl: &Arc<gl::Context>) -> Option<Client> {
        let login_info = LoginInfo {
            username: self.login_file.username.clone(),
            address: self.login_file.last_login_address.clone(),
        };

        // Add to saved logins if not present
        if !self.login_file.addresses.contains(&login_info.address) {
            self.login_file.addresses.push(login_info.address.clone());
        }

        // Save login file
        self.login_file.addresses.sort();
        self.login_file.save().unwrap();

        log::info!(
            "Logging into {} as {}",
            login_info.address,
            login_info.username
        );
        let c = Client::new(gl.clone(), login_info);
        match c {
            Ok(c) => Some(c),
            Err(e) => {
                self.err_text = format!("Error: {:#}", e);
                None
            }
        }
    }

    /// Returns true if a login with the given login_info has been requested
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        ui.label("ChatImproVR login:");

        ui.horizontal(|ui| {
            ui.label("Username: ");
            ui.text_edit_singleline(&mut self.login_file.username);
        });

        let mut ret = false;

        ui.horizontal(|ui| {
            ui.label("Address: ");
            ui.text_edit_singleline(&mut self.login_file.last_login_address);
            ret |= ui.button("Connect").clicked();
        });

        // Error text
        ui.label(RichText::new(&self.err_text).color(Color32::RED));

        ui.separator();
        ui.label("Saved logins:");

        // Login editor
        let mut dup = None;
        let mut del = None;
        for (idx, addr) in self.login_file.addresses.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(addr);
                if ui.button(" + ").clicked() {
                    dup = Some(idx);
                }
                if ui.button(" - ").clicked() {
                    del = Some(idx);
                }

                if ui.button("Connect").clicked() {
                    // Move this into the address bar
                    self.login_file.last_login_address = addr.clone();
                    ret = true;
                }
            });
        }

        if let Some(del) = del {
            self.login_file.addresses.remove(del);
        }

        if let Some(dup) = dup {
            let entry = self.login_file.addresses[dup].clone();
            self.login_file.addresses.insert(dup, entry);
        }

        ret
    }
}
