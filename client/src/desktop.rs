use crate::desktop_input::DesktopInputHandler;
use crate::{project_dirs, Client, LoginFile, LoginInfo, Opt};
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
    let mut login_screen = LoginScreen::new(args.clone())?;

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
                                    let addr = login_screen.address.clone();
                                    login_screen.login_file.save(addr).unwrap();
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

#[derive(Default)]
struct LoginScreen {
    login_file: LoginFile,
    address: String,
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
            address: login_file.last_login_address.clone(),
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

        /*
        ui.horizontal(|ui| {
            ui.label("Address: ");
            ui.text_edit_singleline(&mut self.login_info.address);
        });

        ui.horizontal(|ui| {
            ui.label("Username: ");
            ui.text_edit_singleline(&mut self.login_info.username);
        });
        */

        let ret = ui.button("Connect").clicked();
        ui.label(RichText::new(&self.err_text).color(Color32::RED));

        ret
    }
}
