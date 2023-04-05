use cimvr_common::{
    glam::{Mat3, Mat4, Quat, Vec3},
    render::CameraComponent,
    Transform,
    desktop::{InputEvent,WindowEvent},
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

struct Camera2D {
    proj: Orthographic,
}
make_app_state!(Camera2D, DummyUserState);

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
            screen_size: (1980, 1080),
            left: -10.,
            right: 10.,
            bottom: -10.,
            top: 10.,
            near: 0.1,
            far: 100.,
            proj: [Mat4::IDENTITY; 2],
        }
    }
}

impl Orthographic {
    pub fn update_proj(&mut self, width: f32, height: f32, input: &InputEvent) {
        
        // Check if the screen size changes
        if let InputEvent::Window(WindowEvent::Resized { width: screen_width, height: screen_height }) = input {
            self.screen_size = (*screen_width, *screen_height);
        }

        // Calculate the ratio of the screen size~
        let mut x_ratio = width;
        let mut y_ratio = height;

        while x_ratio / 10. >= 1. {
            x_ratio /= 10.;
        }

        while y_ratio / 10. >= 1. {
            y_ratio /= 10.;
        }

        // Update the ideal projection matrix of the screen
        self.left = self.screen_size.0 as f32 / 2. / -(width / 2.) * x_ratio;
        self.right = self.screen_size.0 as f32 / 2. / (width / 2.) *  x_ratio;
        self.bottom = self.screen_size.1 as f32 / 2. / -(height / 2.) * y_ratio;
        self.top = self.screen_size.1 as f32 / 2. / (height / 2.) * y_ratio;

        dbg!(self.left, self.right, self.bottom, self.top);

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

    pub fn face_towards(&self, dir: Vec3, up: Vec3) -> Quat {
        let zaxis = dir.normalize();
        let xaxis = up.cross(zaxis).normalize();
        let yaxis = zaxis.cross(xaxis).normalize();

        let mat = Mat3::from_cols(xaxis, yaxis, zaxis);

        Quat::from_mat3(&mat)
    }

    pub fn matrices(&self) -> [Mat4; 2] {
        self.proj
    }

    pub fn camera_on_positive_z_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(0., 0., 10.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_custom_axis(&self, pos_x: f32, pos_y: f32, pos_z: f32) -> Transform {
        Transform {
            pos: Vec3::new(pos_x, pos_y, pos_z),
            orient: self.face_towards(Vec3::new(pos_x, pos_y, pos_z), Vec3::Y),
        }
    }
}

impl UserState for Camera2D {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create camera
        io.create_entity()
            .add_component(Transform::identity())
            .add_component(CameraComponent {
                clear_color: [0.; 3],
                projection: Default::default(),
            })
            .build();

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule
            .add_system(Self::update)
            .stage(Stage::PreUpdate)
            .subscribe::<InputEvent>()
            .query::<Transform>(Access::Write)
            .query::<CameraComponent>(Access::Write)
            .build();

        Self {
            proj: Orthographic::new(),
        }
    }
}

impl Camera2D {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        
        for input in io.inbox::<InputEvent>() {

            self.proj.update_proj(10., 20., &input);
        }

        let clear_color = [0.; 3];
        let new_projection = self.proj.matrices();

        for key in query.iter() {
        }

        for key in query.iter() {
            query.write::<CameraComponent>(
                key,
                &CameraComponent {
                    clear_color: clear_color,
                    projection: new_projection,
                },
            );
        }
    }
}
