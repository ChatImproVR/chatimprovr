extern crate glow as gl;
extern crate openxr as xr;

use anyhow::{bail, Context, Result, format_err};
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{Access, QueryComponent, Synchronized};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::network::{
    length_delmit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, Engine};
use desktop::DesktopInputHandler;
use egui_glow::EguiGlow;
use gl::HasContext;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use render::RenderPlugin;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use ui::OverlayUi;

mod desktop;
mod render;
mod ui;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ChatImproVR client",
    about = "Client application for experiencing the ChatImproVR metaverse"
)]
struct Opt {
    /// Remote host address, defaults to local server
    #[structopt(short, long, default_value = "127.0.0.1:5031")]
    connect: SocketAddr,

    /// Plugins
    plugins: Vec<PathBuf>,

    /// Whether to use VR
    #[structopt(long)]
    vr: bool,
}

struct Client {
    engine: Engine,
    render: RenderPlugin,
    recv_buf: AsyncBufferedReceiver,
    conn: TcpStream,
    hotload: Hotloader,
    ui: OverlayUi,
}

fn main() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse args
    let args = Opt::from_args();

    if args.vr {
        virtual_reality(args)
    } else {
        desktop(args)
    }
}

impl Client {
    pub fn new(
        gl: std::sync::Arc<gl::Context>,
        plugins: &[PathBuf],
        addr: SocketAddr,
    ) -> Result<Self> {
        // Connect to remote host
        let conn = TcpStream::connect(addr)?;
        conn.set_nonblocking(true)?;

        // Set up hotloading
        let hotload = Hotloader::new(&plugins)?;

        // Set up engine and initialize plugins
        let mut engine = Engine::new(&plugins, Config { is_server: false })?;

        // Set up rendering
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;

        let ui = OverlayUi::new(&mut engine);

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            hotload,
            conn,
            ui,
            recv_buf: AsyncBufferedReceiver::new(),
            engine,
            render,
        })
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.render.set_screen_size(width, height)
    }

    /// Synchronize with remote and with plugin hotloading
    pub fn download(&mut self) -> Result<()> {
        // Check for hotloaded plugins
        for path in self.hotload.hotload()? {
            log::info!("Reloading {}", path.display());
            self.engine.reload(path)?;
        }

        // Synchronize
        loop {
            match self.recv_buf.read(&mut self.conn)? {
                ReadState::Invalid => {
                    log::error!("Failed to parse invalid message");
                }
                ReadState::Incomplete => break Ok(()),
                ReadState::Disconnected => {
                    bail!("Disconnected");
                }
                ReadState::Complete(buf) => {
                    // Update state!
                    let recv: ServerToClient = deserialize(std::io::Cursor::new(buf))?;
                    for msg in recv.messages {
                        self.engine.broadcast_local(msg);
                    }
                    self.engine.ecs().import(
                        &[QueryComponent::new::<Synchronized>(Access::Read)],
                        recv.ecs,
                    );
                }
            }
        }
    }

    pub fn update_ui(&mut self, ctx: &egui::Context) {
        self.ui.update(&mut self.engine);
        self.ui.run(ctx, &mut self.engine);
    }

    pub fn render_frame(&mut self) -> Result<()> {
        self.render.frame(&mut self.engine)
    }

    pub fn upload(&mut self) -> Result<()> {
        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };
        length_delmit_message(&msg, &mut self.conn)?;
        self.conn.flush()?;

        Ok(())
    }

    fn engine(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

fn desktop(args: Opt) -> Result<()> {
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
    let gl = std::sync::Arc::new(gl);

    // Set up egui
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    // Set up desktop input
    let mut input = DesktopInputHandler::new();

    // Setup client code
    let mut client = Client::new(gl, &args.plugins, args.connect)?;

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
                client.engine().send(input.get_history());

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
                client.render_frame().expect("Frame render");

                // Render UI
                egui_glow.paint(glutin_ctx.window());

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
                if !egui_glow.on_event(&event) {
                    input.handle_winit_event(event);
                }

                match event {
                    WindowEvent::Resized(ph) => {
                        client.set_window_size(ph.width, ph.height);
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
    });
}

fn virtual_reality(args: Opt) -> Result<()> {
    // Load OpenXR from platform-specific location
    #[cfg(target_os = "linux")]
    let entry = unsafe { xr::Entry::load()? };

    #[cfg(target_os = "windows")]
    let entry = xr::Entry::linked();

    /*
    // Application info
    let app_info = xr::ApplicationInfo {
        application_name: "ChatImproVR",
        application_version: 0,
        engine_name: "ChatImproVR",
        engine_version: 0,
    };

    // Ensure we have the OpenGL extension
    let available_extensions = entry.enumerate_extensions()?;
    assert!(available_extensions.khr_opengl_enable);

    // Enable the OpenGL extension
    let mut extensions = xr::ExtensionSet::default();
    extensions.khr_opengl_enable = true;

    // Create instance
    let xr_instance = entry.create_instance(&app_info, &extensions, &[])?;
    let instance_props = xr_instance.properties().unwrap();
    println!(
        "loaded OpenXR runtime: {} {}",
        instance_props.runtime_name, instance_props.runtime_version
    );

    // Get headset system
    let xr_system = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

    let xr_view_configs = xr_instance.enumerate_view_configurations(xr_system)?;
    assert_eq!(xr_view_configs.len(), 1);
    let xr_view_type = xr_view_configs[0];

    let xr_views = xr_instance.enumerate_view_configuration_views(xr_system, xr_view_type)?;

    // Check what blend mode is valid for this device (opaque vs transparent displays). We'll just
    // take the first one available!
    let xr_environment_blend_mode =
        xr_instance.enumerate_environment_blend_modes(xr_system, xr_view_type)?[0];

    // TODO: Check this???
    let _xr_opengl_requirements = xr_instance.graphics_requirements::<xr::OpenGL>(xr_system)?;

    // Create window
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_title("Hello world!")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024.0f32, 768.0));

    let windowed_context = glutin::ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let (ctx, window) = unsafe { windowed_context.split() };
    let ctx = unsafe { ctx.make_current().unwrap() };

    // Load OpenGL
    let gl = unsafe { gl::Context::from_loader_function(|s| ctx.get_proc_address(s) as *const _) };

    let session_create_info = glutin_openxr_opengl_helper::session_create_info(&ctx, &window)?;

    // Setup client code
    let mut client = Client::new(gl, &args.plugins, args.connect)?;

    // Create session
    let (xr_session, mut xr_frame_waiter, mut xr_frame_stream) =
        unsafe { xr_instance.create_session::<xr::OpenGL>(xr_system, &session_create_info)? };

    // Determine swapchain formats
    let xr_swapchain_formats = xr_session.enumerate_swapchain_formats()?;

    let color_swapchain_format = xr_swapchain_formats
        .iter()
        .copied()
        .find(|&f| f == gl::SRGB8_ALPHA8)
        .unwrap_or(xr_swapchain_formats[0]);

    /*
    let depth_swapchain_format = xr_swapchain_formats
        .iter()
        .copied()
        .find(|&f| f == VR_DEPTH_FORMAT)
        .expect("No suitable depth format found");
    */

    // Create color swapchain
    let mut swapchain_color_images = vec![];
    let mut swapchain_depth_images = vec![];
    let mut xr_swapchains = vec![];

    // Set up swapchains and get images
    for &xr_view in &xr_views {
        let image_types = [(), ()];

        let width = xr_view.recommended_image_rect_width;
        let height = xr_view.recommended_image_rect_height;

        let xr_swapchain_create_info = xr::SwapchainCreateInfo::<xr::OpenGL> {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::SAMPLED
                | xr::SwapchainUsageFlags::COLOR_ATTACHMENT,
            format: color_swapchain_format,
            sample_count: xr_view.recommended_swapchain_sample_count,
            width,
            height,
            face_count: 1,
            array_size: 1,
            mip_count: 1,
        };

        let xr_swapchain = xr_session.create_swapchain(&xr_swapchain_create_info)?;

        let color_images: Vec<gl::NativeTexture> = xr_swapchain
            .enumerate_images()?
            .into_iter()
            .map(|tex| unsafe { gl::Context::create_texture_from_gl_name(tex) })
            .collect();

        let mut depth_images = vec![];

        for _ in &color_images {
            depth_images.push(get_vr_depth_texture(&gl, width as i32, height as i32).unwrap());
        }

        swapchain_depth_images.push(depth_images);
        swapchain_color_images.push(color_images);
        xr_swapchains.push(xr_swapchain);
    }

    // Create OpenGL framebuffers
    let mut gl_framebuffers = vec![];
    for _ in &xr_views {
        let fb = unsafe {
            gl.create_framebuffer()
                .map_err(|s| format_err!("Failed to create framebuffer; {}", s))?
        };

        gl_framebuffers.push(fb);
    }

    // Compile shaders
    let xr_play_space =
        xr_session.create_reference_space(xr::ReferenceSpaceType::LOCAL, xr::Posef::IDENTITY)?;

    let mut xr_event_buf = xr::EventDataBuffer::default();

    'main: loop {
        // Handle OpenXR Events
        while let Some(event) = xr_instance.poll_event(&mut xr_event_buf)? {
            match event {
                xr::Event::InstanceLossPending(_) => break 'main,
                xr::Event::SessionStateChanged(delta) => {
                    match delta.state() {
                        xr::SessionState::IDLE | xr::SessionState::UNKNOWN => {
                            continue 'main;
                        }
                        //xr::SessionState::FOCUSED | xr::SessionState::SYNCHRONIZED | xr::SessionState::VISIBLE => (),
                        xr::SessionState::STOPPING => {
                            xr_session.end()?;
                            break 'main;
                        }
                        xr::SessionState::LOSS_PENDING | xr::SessionState::EXITING => {
                            // ???
                        }
                        xr::SessionState::READY => {
                            dbg!(delta.state());
                            xr_session.begin(xr_view_type)?;
                        }
                        _ => continue 'main,
                    }
                }
                _ => (),
            }
        }

        // --- Wait for our turn to do head-pose dependent computation and render a frame
        let xr_frame_state = xr_frame_waiter.wait()?;

        // Signal to OpenXR that we are beginning graphics work
        xr_frame_stream.begin()?;

        // Early exit
        if !xr_frame_state.should_render {
            xr_frame_stream.end(
                xr_frame_state.predicted_display_time,
                xr_environment_blend_mode,
                &[],
            )?;
            continue;
        }

        // Get head positions from server
        let state = client.update_heads()?;
        let head_mats = head_matrices(&state.heads);
        engine.update_heads(&gl, &head_mats);

        // Get OpenXR Views
        // TODO: Do this as close to render-time as possible!!
        let (_xr_view_state_flags, xr_view_poses) = xr_session.locate_views(
            xr_view_type,
            xr_frame_state.predicted_display_time,
            &xr_play_space,
        )?;

        for view_idx in 0..xr_views.len() {
            // Acquire image
            let xr_swapchain_img_idx = xr_swapchains[view_idx].acquire_image()?;
            xr_swapchains[view_idx].wait_image(xr::Duration::from_nanos(1_000_000_000_000))?;

            // Bind framebuffer
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(gl_framebuffers[view_idx]));

            // Set scissor and viewport
            let view = xr_views[view_idx];
            let w = view.recommended_image_rect_width as i32;
            let h = view.recommended_image_rect_height as i32;

            // Set the texture as the render target
            let img_idx = xr_swapchain_img_idx as usize;
            let color_texture = swapchain_color_images[view_idx][img_idx];
            let depth_texture = swapchain_depth_images[view_idx][img_idx];

            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(color_texture),
                0,
            );

            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                Some(depth_texture),
                0,
            );

            // Set view and projection matrices
            let headset_view = xr_view_poses[view_idx];

            let view = view_from_pose(&headset_view.pose);
            let proj = projection_from_fov(&headset_view.fov, 0.01, 1000.);

            engine.frame(&gl, proj, view).expect("Engine error");

            // Unbind framebuffer
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);

            // Release image
            xr_swapchains[view_idx].release_image()?;
        }

        // Set up projection views
        let mut xr_projection_views = vec![];
        for view_idx in 0..xr_views.len() {
            // Set up projection view
            let xr_sub_image = xr::SwapchainSubImage::<xr::OpenGL>::new()
                .swapchain(&xr_swapchains[view_idx])
                .image_array_index(0)
                .image_rect(xr::Rect2Di {
                    offset: xr::Offset2Di { x: 0, y: 0 },
                    extent: xr::Extent2Di {
                        width: xr_views[view_idx].recommended_image_rect_width as i32,
                        height: xr_views[view_idx].recommended_image_rect_height as i32,
                    },
                });

            let xr_proj_view = xr::CompositionLayerProjectionView::<xr::OpenGL>::new()
                .pose(xr_view_poses[view_idx].pose)
                .fov(xr_view_poses[view_idx].fov)
                .sub_image(xr_sub_image);

            xr_projection_views.push(xr_proj_view);
        }

        let layers = xr::CompositionLayerProjection::new()
            .space(&xr_play_space)
            .views(&xr_projection_views);

        xr_frame_stream.end(
            xr_frame_state.predicted_display_time,
            xr_environment_blend_mode,
            &[&layers],
        )?;

        // Update head position in server. This is done after all the display work, so that we
        // don't introduce latency
        let state = ClientState {
            head: head_from_xr_pose(&xr_view_poses[0].pose),
        };
        client.send_state(&state)?;
    }
    */

    Ok(())
}

fn get_vr_depth_texture(
    gl: &gl::Context,
    width: i32,
    height: i32,
) -> Result<gl::NativeTexture, String> {
    /*
    unsafe {
        let depth_tex = gl.create_texture()?;
        gl.bind_texture(gl::TEXTURE_2D, Some(depth_tex));
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            VR_DEPTH_FORMAT as _,
            width,
            height,
            0,
            gl::DEPTH_COMPONENT,
            gl::UNSIGNED_INT,
            None,
        );

        Ok(depth_tex)
    }
    */
    todo!()
}
