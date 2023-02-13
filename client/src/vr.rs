use crate::{Client, Opt};
use anyhow::{format_err, Result};
use cimvr_common::vr::{VrFov, VrUpdate};
use cimvr_common::Transform;
use cimvr_engine::interface::system::Stage;
use gl::HasContext;
use nalgebra::{Point3, Quaternion, Unit};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use xr::View;

const VR_DEPTH_FORMAT: u32 = gl::DEPTH_COMPONENT24;

pub fn mainloop(args: Opt) -> Result<()> {
    let mut main = MainLoop::new(args.plugins, args.connect)?;
    while main.frame()? {}

    Ok(())
}

struct MainLoop {
    client: Client,
    gl: Arc<gl::Context>,
    gl_framebuffers: Vec<gl::NativeFramebuffer>,
    xr_frame_stream: xr::FrameStream<xr::OpenGL>,
    xr_instance: xr::Instance,
    xr_session: xr::Session<xr::OpenGL>,
    xr_event_buf: xr::EventDataBuffer,
    xr_view_type: xr::ViewConfigurationType,
    xr_frame_waiter: xr::FrameWaiter,
    xr_environment_blend_mode: xr::EnvironmentBlendMode,
    xr_play_space: xr::Space,
    xr_views: Vec<xr::ViewConfigurationView>,
    xr_swapchains: Vec<xr::Swapchain<xr::OpenGL>>,
    swapchain_color_images: Vec<Vec<gl::NativeTexture>>,
    swapchain_depth_images: Vec<Vec<gl::NativeTexture>>,
    _glutin_ctx: glutin::ContextWrapper<glutin::PossiblyCurrent, ()>,
    _glutin_window: glutin::window::Window,
    plugin_interface: PluginVrInterfacing,
}

impl MainLoop {
    pub fn new(plugins: Vec<PathBuf>, connect: SocketAddr) -> Result<Self> {
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
        log::info!(
            "loaded OpenXR runtime: {} {}",
            instance_props.runtime_name,
            instance_props.runtime_version
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
        let window_builder = glutin::window::WindowBuilder::new().with_title("ChatImproVR");

        let windowed_context = glutin::ContextBuilder::new()
            .build_windowed(window_builder, &event_loop)
            .unwrap();

        let (ctx, glutin_window) = unsafe { windowed_context.split() };
        let glutin_ctx = unsafe { ctx.make_current().unwrap() };

        // Load OpenGL
        let gl = unsafe {
            gl::Context::from_loader_function(|s| glutin_ctx.get_proc_address(s) as *const _)
        };
        let gl = Arc::new(gl);

        let session_create_info =
            glutin_openxr_opengl_helper::session_create_info(&glutin_ctx, &glutin_window)?;

        // Setup client code
        let client = Client::new(gl.clone(), &plugins, connect)?;

        // Create session
        let (xr_session, xr_frame_waiter, xr_frame_stream) =
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

        // Get the floor space
        let xr_play_space = xr_session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)?;

        let xr_event_buf = xr::EventDataBuffer::default();

        let plugin_interface = PluginVrInterfacing::new(&xr_instance, &xr_session)?;

        Ok(Self {
            client,
            gl,
            gl_framebuffers,
            xr_frame_stream,
            xr_instance,
            xr_session,
            xr_event_buf,
            xr_view_type,
            xr_frame_waiter,
            xr_environment_blend_mode,
            xr_play_space,
            xr_swapchains,
            xr_views,
            swapchain_color_images,
            swapchain_depth_images,
            _glutin_ctx: glutin_ctx,
            _glutin_window: glutin_window,
            plugin_interface,
        })
    }

    pub fn frame(&mut self) -> Result<bool> {
        // Handle OpenXR Events
        while let Some(event) = self.xr_instance.poll_event(&mut self.xr_event_buf)? {
            match event {
                xr::Event::InstanceLossPending(_) => return Ok(false),
                xr::Event::SessionStateChanged(delta) => match delta.state() {
                    xr::SessionState::STOPPING => {
                        self.xr_session.end()?;
                        return Ok(false);
                    }
                    xr::SessionState::READY => {
                        self.xr_session.begin(self.xr_view_type)?;
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        // --- Wait for our turn to do head-pose dependent computation and render a frame
        let xr_frame_state = self.xr_frame_waiter.wait()?;

        // Signal to OpenXR that we are beginning graphics work
        self.xr_frame_stream.begin()?;

        // Early exit
        if !xr_frame_state.should_render {
            self.xr_frame_stream.end(
                xr_frame_state.predicted_display_time,
                self.xr_environment_blend_mode,
                &[],
            )?;
            return Ok(true);
        }

        // Download messages from server
        self.client.download().expect("Message download");

        // Get gamepad state
        let gamepad_state = self.client.gamepad.update();
        self.client.engine().send(gamepad_state);

        // Pre update stage
        self.client
            .engine()
            .dispatch(Stage::PreUpdate)
            .expect("Frame pre-update");

        // Get OpenXR Views
        let (_xr_view_state_flags, xr_view_poses) = self.xr_session.locate_views(
            self.xr_view_type,
            xr_frame_state.predicted_display_time,
            &self.xr_play_space,
        )?;

        // Send update data to plugins
        let vr_update = self.plugin_interface.update(
            &xr_view_poses,
            &self.xr_session,
            &xr_frame_state,
            &self.xr_play_space,
        )?;
        self.client.engine().send(vr_update);

        // Update stage
        self.client
            .engine()
            .dispatch(Stage::Update)
            .expect("Frame udpate");

        // Get OpenXR Views
        // TODO: Do this as close to render-time as possible!!
        let (_xr_view_state_flags, xr_view_poses) = self.xr_session.locate_views(
            self.xr_view_type,
            xr_frame_state.predicted_display_time,
            &self.xr_play_space,
        )?;

        for view_idx in 0..self.xr_views.len() {
            // Acquire image
            let xr_swapchain_img_idx = self.xr_swapchains[view_idx].acquire_image()?;
            self.xr_swapchains[view_idx].wait_image(xr::Duration::from_nanos(1_000_000_000_000))?;

            // Bind framebuffer
            unsafe {
                self.gl
                    .bind_framebuffer(gl::FRAMEBUFFER, Some(self.gl_framebuffers[view_idx]));
            }

            // Set scissor and viewport
            let view = self.xr_views[view_idx];
            self.client.set_resolution(
                view.recommended_image_rect_width,
                view.recommended_image_rect_height,
            );

            // Set the texture as the render target
            let img_idx = xr_swapchain_img_idx as usize;
            let color_texture = self.swapchain_color_images[view_idx][img_idx];
            let depth_texture = self.swapchain_depth_images[view_idx][img_idx];

            unsafe {
                self.gl.framebuffer_texture_2d(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    Some(color_texture),
                    0,
                );

                self.gl.framebuffer_texture_2d(
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
            self.client
                .render_frame(transf.view(), view_idx)
                .expect("Frame render");

            // Unbind framebuffer
            unsafe {
                self.gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            }

            // Release image
            self.xr_swapchains[view_idx].release_image()?;
        }

        // Set up projection views
        let mut xr_projection_views = vec![];
        for view_idx in 0..self.xr_views.len() {
            // Set up projection view
            let xr_sub_image = xr::SwapchainSubImage::<xr::OpenGL>::new()
                .swapchain(&self.xr_swapchains[view_idx])
                .image_array_index(0)
                .image_rect(xr::Rect2Di {
                    offset: xr::Offset2Di { x: 0, y: 0 },
                    extent: xr::Extent2Di {
                        width: self.xr_views[view_idx].recommended_image_rect_width as i32,
                        height: self.xr_views[view_idx].recommended_image_rect_height as i32,
                    },
                });

            let xr_proj_view = xr::CompositionLayerProjectionView::<xr::OpenGL>::new()
                .pose(xr_view_poses[view_idx].pose)
                .fov(xr_view_poses[view_idx].fov)
                .sub_image(xr_sub_image);

            xr_projection_views.push(xr_proj_view);
        }

        let layers = xr::CompositionLayerProjection::new()
            .space(&self.xr_play_space)
            .views(&xr_projection_views);

        self.xr_frame_stream.end(
            xr_frame_state.predicted_display_time,
            self.xr_environment_blend_mode,
            &[&layers],
        )?;

        // Post update stage
        self.client
            .engine()
            .dispatch(Stage::PostUpdate)
            .expect("Frame post-update");

        // Upload messages to server
        self.client.upload().expect("Message upload");

        Ok(true)
    }
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

/// Plugin interfacing for VR controllers, headset
struct PluginVrInterfacing {
    action_set: xr::ActionSet,
    grip_left: xr::Space,
    grip_right: xr::Space,
    aim_left: xr::Space,
    aim_right: xr::Space,

    grip_left_action: xr::Action<xr::Posef>,
    grip_right_action: xr::Action<xr::Posef>,
    aim_left_action: xr::Action<xr::Posef>,
    aim_right_action: xr::Action<xr::Posef>,
}

impl PluginVrInterfacing {
    pub fn new(xr_instance: &xr::Instance, xr_session: &xr::Session<xr::OpenGL>) -> Result<Self> {
        // Create action set
        let action_set = xr_instance.create_action_set("gameplay", "Gameplay", 0)?;

        let grip_left_action =
            action_set.create_action::<xr::Posef>("grip_left", "grip_left", &[])?;

        let aim_left_action = action_set.create_action::<xr::Posef>("aim_left", "aim_left", &[])?;

        let grip_right_action =
            action_set.create_action::<xr::Posef>("grip_right", "grip_right", &[])?;

        let aim_right_action =
            action_set.create_action::<xr::Posef>("aim_right", "aim_right", &[])?;

        xr_instance
            .suggest_interaction_profile_bindings(
                xr_instance
                    .string_to_path("/interaction_profiles/khr/simple_controller")
                    .unwrap(),
                &[
                    xr::Binding::new(
                        &aim_right_action,
                        xr_instance
                            .string_to_path("/user/hand/right/input/aim/pose")
                            .unwrap(),
                    ),
                    xr::Binding::new(
                        &aim_left_action,
                        xr_instance
                            .string_to_path("/user/hand/left/input/aim/pose")
                            .unwrap(),
                    ),
                    xr::Binding::new(
                        &grip_right_action,
                        xr_instance
                            .string_to_path("/user/hand/right/input/grip/pose")
                            .unwrap(),
                    ),
                    xr::Binding::new(
                        &grip_left_action,
                        xr_instance
                            .string_to_path("/user/hand/left/input/grip/pose")
                            .unwrap(),
                    ),
                ],
            )
            .unwrap();

        xr_session.attach_action_sets(&[&action_set])?;

        let aim_right = aim_right_action.create_space(
            xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;

        let aim_left = aim_left_action.create_space(
            xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;

        let grip_right = grip_right_action.create_space(
            xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;

        let grip_left = grip_left_action.create_space(
            xr_session.clone(),
            xr::Path::NULL,
            xr::Posef::IDENTITY,
        )?;

        Ok(Self {
            action_set,
            aim_left,
            aim_right,
            grip_left,
            grip_right,

            aim_left_action,
            aim_right_action,
            grip_left_action,
            grip_right_action,
        })
    }

    pub fn update(
        &mut self,
        views: &[View],
        xr_session: &xr::Session<xr::OpenGL>,
        xr_frame_state: &xr::FrameState,
        stage: &xr::Space,
    ) -> Result<VrUpdate> {
        xr_session.sync_actions(&[xr::ActiveActionSet::new(&self.action_set)])?;

        // Get hand positions
        let aim_right = self
            .aim_right
            .locate(&stage, xr_frame_state.predicted_display_time)?;
        let aim_right = self
            .aim_right_action
            .is_active(xr_session, xr::Path::NULL)?
            .then(|| transform_from_pose(&aim_right.pose));

        let aim_left = self
            .aim_left
            .locate(&stage, xr_frame_state.predicted_display_time)?;
        let aim_left = self
            .aim_left_action
            .is_active(xr_session, xr::Path::NULL)?
            .then(|| transform_from_pose(&aim_left.pose));

        let grip_right = self
            .grip_right
            .locate(&stage, xr_frame_state.predicted_display_time)?;
        let grip_right = self
            .grip_right_action
            .is_active(xr_session, xr::Path::NULL)?
            .then(|| transform_from_pose(&grip_right.pose));

        let grip_left = self
            .grip_left
            .locate(&stage, xr_frame_state.predicted_display_time)?;
        let grip_left = self
            .grip_left_action
            .is_active(xr_session, xr::Path::NULL)?
            .then(|| transform_from_pose(&grip_left.pose));

        // Get VR data for Update stage
        Ok(VrUpdate {
            view_left: transform_from_pose(&views[0].pose),
            view_right: transform_from_pose(&views[1].pose),
            fov_left: convert_fov(&views[0].fov),
            fov_right: convert_fov(&views[1].fov),
            grip_left,
            grip_right,
            aim_left,
            aim_right,
        })
    }
}
