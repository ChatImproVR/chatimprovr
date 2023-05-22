use crate::desktop_input::DesktopInputHandler;
use crate::glutin_window_ctx::create_display;
use crate::{Client, Opt};
use anyhow::Result;
use cimvr_common::glam::Mat4;
use cimvr_engine::interface::system::Stage;
use egui_winit::winit;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use std::sync::Arc;

pub fn mainloop(args: Opt) -> Result<()> {
    // Set up window
    let event_loop = winit::event_loop::EventLoop::new();

    // Set up OpenGL
    let (glutin_ctx, gl) = create_display(&event_loop);
    let gl = Arc::new(gl);

    // Set up egui
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone(), None);

    // Set up desktop input
    let mut input = DesktopInputHandler::new();

    // Setup client code
    let mut client = Client::new(gl, &args.plugins, args.connect, args.username.unwrap())?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
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

                // Render UI
                egui_glow.paint(glutin_ctx.window());

                // Show the window on first draw 
                glutin_ctx.window().set_visible(true);

                // Post update stage
                client
                    .engine()
                    .dispatch(Stage::PostUpdate)
                    .expect("Frame post-update");

                // Upload messages to server
                client.upload().expect("Message upload");

                glutin_ctx.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => {
                if !egui_glow.on_event(&event).consumed {
                    input.handle_winit_event(event);
                }

                match event {
                    WindowEvent::Resized(ph) => {
                        client.set_resolution(ph.width, ph.height);
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
