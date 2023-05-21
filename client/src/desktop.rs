use crate::desktop_input::DesktopInputHandler;
use crate::{project_dirs, Client, Opt};
use anyhow::Result;
use cimvr_common::glam::Mat4;
use cimvr_engine::interface::system::Stage;
use directories::ProjectDirs;
use egui::{Color32, DragValue, Label, RichText, Ui};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;

pub fn mainloop(mut args: Opt) -> Result<()> {
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

    // Setup client code
    let mut client: Option<Client> = None;
    let mut login_screen = LoginScreen::new()?;
    if let Some(user) = &args.username {
        login_screen.login_info.username = user.clone();
    }

    if let Some(addr) = &args.connect {
        login_screen.login_info.address = addr.clone();
    }

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
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
                                if client.is_some() {
                                    login_screen.login_info.save().unwrap();
                                }
                            }
                        });
                    });
                }

                if let Some(client) = &mut client {
                    // Download messages from server
                    client.download().expect("Message download");

                    // Send input history
                    input.get_history(client.engine());
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

                    // Collect UI input
                    egui_glow.run(glutin_ctx.window(), |ctx| client.update_ui(ctx));

                    // Render frame
                    client
                        .render_frame(Mat4::IDENTITY, 0)
                        .expect("Frame render");
                }

                // Render UI
                egui_glow.paint(glutin_ctx.window());

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
                if let Some(travel_request) = client.as_mut().and_then(|c| c.travel_request()) {
                    // TODO: Handle port here?
                    login_screen.login_info.address = travel_request.address;
                    // TODO: This doesn't report errors
                    client = login_screen.login(&gl);
                }

                glutin_ctx.swap_buffers().unwrap();
            }
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

struct LoginInfo {
    address: String,
    username: String,
}

impl LoginInfo {
    fn config_path() -> PathBuf {
        let proj = project_dirs();
        if !proj.config_dir().is_dir() {
            std::fs::create_dir_all(proj.config_dir()).unwrap();
        }
        proj.config_dir().join("login.conf")
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if !config_path.is_file() {
            Ok(Self::default())
        } else {
            let text = std::fs::read_to_string(config_path)?;
            let mut lines = text.lines();
            Ok(Self {
                address: lines.next().unwrap().to_string(),
                username: lines.next().unwrap().to_string(),
            })
        }
    }

    /// Returns the address assigned, with the default port appended if not present
    pub fn addr_with_port(&self) -> String {
        let addr = self.address.clone();
        if addr.contains(':') {
            addr
        } else {
            addr + ":5031"
        }
    }

    pub fn save(&self) -> Result<()> {
        use std::fmt::Write;
        let mut s = String::new();
        writeln!(s, "{}", self.address)?;
        writeln!(s, "{}", self.username)?;
        std::fs::write(Self::config_path(), s)?;
        Ok(())
    }
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            username: "Anon".to_string(),
        }
    }
}

#[derive(Default)]
struct LoginScreen {
    login_info: LoginInfo,
    err_text: String,
}

impl LoginScreen {
    pub fn new() -> Result<Self> {
        Ok(Self {
            login_info: LoginInfo::load()?,
            err_text: "".into(),
        })
    }

    /// Takes gl as an argument in order to create client instance (nothing else!)
    pub fn login(&mut self, gl: &Arc<gl::Context>) -> Option<Client> {
        log::info!("Logging into {} as {}", self.login_info.addr_with_port(), self.login_info.username);
        let c = Client::new(gl.clone(), self.login_info.addr_with_port(), self.login_info.username.clone());
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
            ui.label("Address: ");
            ui.text_edit_singleline(&mut self.login_info.address);
        });

        ui.horizontal(|ui| {
            ui.label("Username: ");
            ui.text_edit_singleline(&mut self.login_info.username);
        });

        let ret = ui.button("Connect").clicked();
        ui.label(RichText::new(&self.err_text).color(Color32::RED));

        ret
    }
}
