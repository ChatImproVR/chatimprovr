use crate::desktop_input::DesktopInputHandler;
use crate::{project_dirs, Client, Opt};
use anyhow::Result;
use cimvr_common::glam::Mat4;
use cimvr_engine::interface::system::Stage;
use directories::ProjectDirs;
use egui::{Color32, DragValue, Label, RichText};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;

pub fn mainloop(args: Opt) -> Result<()> {
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
    let mut login_info = LoginInfo::load()?;
    let mut err_text = "".to_string();

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
                    egui_glow.run(glutin_ctx.window(), |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label("ChatImproVR login:");

                            ui.horizontal(|ui| {
                                ui.label("Address: ");
                                ui.text_edit_singleline(&mut login_info.address);
                                ui.add(
                                    DragValue::new(&mut login_info.port)
                                        .prefix("Port: ")
                                        .clamp_range(1..=u16::MAX as _),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Username: ");
                                ui.text_edit_singleline(&mut login_info.username);
                            });

                            if ui.button("Connect").clicked() {
                                let full_addr =
                                    format!("{}:{}", login_info.address, login_info.port);
                                log::info!("Logging into {} as {}", full_addr, login_info.username);
                                let c =
                                    Client::new(gl.clone(), full_addr, login_info.username.clone());
                                match c {
                                    Ok(c) => {
                                        client = Some(c);
                                        login_info.save();
                                    }
                                    Err(e) => err_text = format!("Error: {:#}", e),
                                }
                            }
                            ui.label(RichText::new(&err_text).color(Color32::RED));
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

struct LoginInfo {
    address: String,
    port: u16,
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
                port: lines.next().unwrap().parse().unwrap(),
                username: lines.next().unwrap().to_string(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        use std::fmt::Write;
        let mut s = String::new();
        writeln!(s, "{}", self.address)?;
        writeln!(s, "{}", self.port)?;
        writeln!(s, "{}", self.username)?;
        std::fs::write(Self::config_path(), s)?;
        Ok(())
    }
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 5031,
            username: "Anon".to_string(),
        }
    }
}
