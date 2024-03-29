use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::{
    desktop::{InputEvent, WindowEvent},
    vr::{VrFov, VrUpdate},
    Transform,
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
    pub fn handle_event(&mut self, input: &InputEvent) {
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

pub struct Orthographic {
    screen_size: (u32, u32),
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    proj: [Mat4; 2],
}

impl Default for Orthographic {
    fn default() -> Self {
        Self {
            screen_size: (1920, 1080),
            left: -10.,
            right: 10.,
            bottom: -10.,
            top: 10.,
            near: -100.,
            far: 100.,
            proj: [Mat4::IDENTITY; 2],
        }
    }
}

impl Orthographic {
    pub fn update_proj(&mut self, mut width: f32, mut height: f32, input: &InputEvent) {
        // Check if the screen size changes
        if let InputEvent::Window(WindowEvent::Resized {
            width: screen_width,
            height: screen_height,
        }) = input
        {
            self.screen_size = (*screen_width, *screen_height);
        }

        // Ge the aspect ratio
        let aspect_ratio = self.screen_size.0 as f32 / self.screen_size.1 as f32;

        // If the world aspect ratio is biggeer or equal to the screen size, then update the height that matches the screen aspect ratio
        if width / height >= aspect_ratio as f32 {
            height = width / aspect_ratio as f32;
        }
        // Otherwise, update the width screen aspect ratio oto the world aspect ratio
        else {
            width = height * aspect_ratio as f32;
        }

        // Get the correct values for setting up the orthographic arguments
        self.left = -width / 2.;
        self.right = width / 2.;
        self.bottom = -height / 2.;
        self.top = height / 2.;

        // Recreate the new projection matrix based on the updated screen size
        let new_proj = Mat4::orthographic_rh_gl(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        self.proj = [new_proj; 2];
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn matrices(&self) -> [Mat4; 2] {
        self.proj
    }

    pub fn camera_on_positive_z_axis(&self) -> Transform {
        // This camera control is for 2D arcade games that are played on a flat screen: z-axis is to us
        Transform {
            pos: Vec3::new(0., 0., 5.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_custom_axis(
        &self,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        degree_x: f32,
        degree_y: f32,
        degree_z: f32,
    ) -> Transform {
        Transform {
            pos: Vec3::new(pos_x, pos_y, pos_z),
            orient: Quat::from_euler(
                EulerRot::XYZ,
                degree_x.to_radians(),
                degree_y.to_radians(),
                degree_z.to_radians(),
            ),
        }
    }
}
