use crate::desktop_input::DesktopInputHandler;
use crate::{Client, Opt};
use anyhow::Result;
use cimvr_common::glam::Mat4;
use cimvr_engine::interface::system::Stage;
use egui::DragValue;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
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
    let mut input_username = "Anon".to_string();
    let mut input_port = 5031;
    let mut input_addr = "127.0.0.1".to_string();

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
                                ui.text_edit_singleline(&mut input_addr);
                                ui.add(
                                    DragValue::new(&mut input_port)
                                        .prefix("Port: ")
                                        .clamp_range(1..=u16::MAX as _),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label("Username: ");
                                ui.text_edit_singleline(&mut input_username);
                            });

                            if ui.button("Connect").clicked() {
                                let full_addr = format!("{input_addr}:{input_port}");
                                log::info!("Logging into {} as {}", full_addr, input_username);
                                let c = Client::new(gl.clone(), full_addr, input_username.clone())
                                    .expect("Failed to create client");
                                client = Some(c);
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
