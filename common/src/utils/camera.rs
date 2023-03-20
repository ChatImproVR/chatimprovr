use glam::Mat4;

use crate::{
    desktop::{InputEvent, WindowEvent},
    vr::{VrFov, VrUpdate},
};

/// Perspective camera matrix utility
/// In desktop mode, makes sure the matrix has the correct aspect ratio
/// In VR mode, makes sure the perspective matrices match the XrFov specification
pub struct Perspective {
    screen_size: (u32, u32),
    pub near_plane: f32,
    pub far_plane: f32,
    pub fov: f32,
    proj: [Mat4; 2],
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            screen_size: (1920, 1080),
            near_plane: 0.01,
            far_plane: 1000.,
            fov: 45_f32.to_radians(),
            proj: [Mat4::IDENTITY; 2],
        }
    }
}

impl Perspective {
    /// Initialize the default perspective
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the left and right perpective matrices, respectively.
    /// In desktop mode these matrices will be equal
    pub fn matrices(&self) -> [Mat4; 2] {
        self.proj
    }

    /// Returns and appropriate perspective matrix matching the size of the window
    pub fn handle_input_events(&mut self, input: &InputEvent) {
        // Handle input events for desktop mode
        // let InputEvents(events) = input;
        if let InputEvent::Window(WindowEvent::Resized { width, height }) = input {
            self.screen_size = (*width, *height);
        }

        // Get projection matrix
        let proj = Mat4::perspective_rh_gl(
            self.fov,
            self.screen_size.0 as f32 / self.screen_size.1 as f32,
            self.near_plane,
            self.far_plane,
        );

        self.proj = [proj; 2];
    }

    pub fn handle_vr_update(&mut self, update: &VrUpdate) {
        self.proj = [update.headset.left.proj, update.headset.right.proj]
            .map(|fov| vr_projection_from_fov(fov, self.near_plane, self.far_plane));
    }
}

/// Creates a projection matrix for the given fov
pub fn vr_projection_from_fov(fov: VrFov, near: f32, far: f32) -> Mat4 {
    // See https://gitlab.freedesktop.org/monado/demos/openxr-simple-example/-/blob/master/main.c
    // XrMatrix4x4f_CreateProjectionFov()

    let tan_left = fov.angle_left.tan();
    let tan_right = fov.angle_right.tan();

    let tan_up = fov.angle_up.tan();
    let tan_down = fov.angle_down.tan();

    let tan_width = tan_right - tan_left;
    let tan_height = tan_up - tan_down;

    let a11 = 2.0 / tan_width;
    let a22 = 2.0 / tan_height;

    let a31 = (tan_right + tan_left) / tan_width;
    let a32 = (tan_up + tan_down) / tan_height;

    let a33 = -far / (far - near);

    let a43 = -(far * near) / (far - near);

    Mat4::from_cols_array_2d(&[
        [a11, 0.0, 0.0, 0.0],
        [0.0, a22, 0.0, 0.0],
        [a31, a32, a33, -1.0],
        [0.0, 0.0, a43, 0.0],
    ])
}
