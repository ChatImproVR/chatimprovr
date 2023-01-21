use anyhow::{format_err, Result};
use cimvr_common::vr::{VrFov, VrUpdate};
use cimvr_common::Transform;
use cimvr_engine::interface::system::Stage;
use gl::HasContext;
use nalgebra::{Point3, Quaternion, Unit};
use std::sync::Arc;
use crate::{Client, Opt};

const VR_DEPTH_FORMAT: u32 = gl::DEPTH_COMPONENT24;

pub fn mainloop(args: Opt) -> Result<()> {
    // Load OpenXR from platform-specific location
    #[cfg(target_os = "linux")]
    let entry = unsafe { xr::Entry::load()? };

    #[cfg(target_os = "windows")]
    let entry = xr::Entry::linked();

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
    let gl = Arc::new(gl);

    let session_create_info = glutin_openxr_opengl_helper::session_create_info(&ctx, &window)?;

    // Setup client code
    let mut client = Client::new(gl.clone(), &args.plugins, args.connect)?;

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

        // Download messages from server
        client.download().expect("Message download");

        // Pre update stage
        client
            .engine()
            .dispatch(Stage::PreUpdate)
            .expect("Frame pre-update");

        // Get OpenXR Views
        let (_xr_view_state_flags, xr_view_poses) = xr_session.locate_views(
            xr_view_type,
            xr_frame_state.predicted_display_time,
            &xr_play_space,
        )?;

        // Get VR data for Update stage
        let vr_data: VrUpdate = VrUpdate {
            left_view: transform_from_pose(&xr_view_poses[0].pose),
            right_view: transform_from_pose(&xr_view_poses[1].pose),
            left_fov: convert_fov(&xr_view_poses[0].fov),
            right_fov: convert_fov(&xr_view_poses[1].fov),
            // TODO
            //right_hand: Transform::identity(),
            //left_hand: Transform::identity(),
            //events: vec![],
        };
        client.engine().send(vr_data);

        // Update stage
        client
            .engine()
            .dispatch(Stage::Update)
            .expect("Frame udpate");

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
            unsafe {
                gl.bind_framebuffer(gl::FRAMEBUFFER, Some(gl_framebuffers[view_idx]));
            }

            // Set scissor and viewport
            let view = xr_views[view_idx];
            client.set_resolution(
                view.recommended_image_rect_width,
                view.recommended_image_rect_height,
            );

            // Set the texture as the render target
            let img_idx = xr_swapchain_img_idx as usize;
            let color_texture = swapchain_color_images[view_idx][img_idx];
            let depth_texture = swapchain_depth_images[view_idx][img_idx];

            unsafe {
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
            }

            // Set view and projection matrices
            let headset_view = xr_view_poses[view_idx];
            let transf = transform_from_pose(&headset_view.pose);

            // Render frame
            client
                .render_frame(transf.view(), view_idx)
                .expect("Frame render");

            // Unbind framebuffer
            unsafe {
                gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            }

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

        // Post update stage
        client
            .engine()
            .dispatch(Stage::PostUpdate)
            .expect("Frame post-update");

        // Upload messages to server
        client.upload().expect("Message upload");
    }

    Ok(())
}

fn get_vr_depth_texture(
    gl: &gl::Context,
    width: i32,
    height: i32,
) -> Result<gl::NativeTexture, String> {
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
}

fn transform_from_pose(pose: &xr::Posef) -> Transform {
    // Convert the rotation quaternion from OpenXR to nalgebra
    let orient = pose.orientation;
    let orient = Quaternion::new(orient.w, orient.x, orient.y, orient.z);
    let orient = Unit::try_new(orient, 0.0).expect("Not a unit orienternion");

    // Convert the position vector from OpenXR to nalgebra
    let pos = pose.position;
    let pos = Point3::new(pos.x, pos.y, pos.z);

    Transform { pos, orient }
}

fn convert_fov(fov: &xr::Fovf) -> VrFov {
    VrFov {
        angle_left: fov.angle_left,
        angle_right: fov.angle_right,
        angle_up: fov.angle_up,
        angle_down: fov.angle_down,
    }
}
