use cimvr_common::{
    glam::{Mat3, Mat4, Quat, Vec3},
    render::CameraComponent,
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

struct Camera2D {
    proj: Orthographic,
}
make_app_state!(Camera2D, DummyUserState);

pub struct Orthographic {
    // screen_size: (u32, u32),
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
            // screen_size: (1980, 1080),
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
    pub fn update_proj(&mut self) {
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

    pub fn camera_on_positive_x_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(10., 0., 0.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_positive_y_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(0., 10., 0.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_positive_z_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(0., 0., 10.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_negative_x_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(-10., 0., 0.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_negative_y_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(0., -10., 0.),
            orient: Default::default(),
        }
    }

    pub fn camera_on_negative_z_axis(&self) -> Transform {
        Transform {
            pos: Vec3::new(0., 0., -10.),
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
        self.proj.update_proj();

        let clear_color = [0.; 3];
        let new_projection = self.proj.matrices();

        for key in query.iter() {
            query.write::<Transform>(key, &self.proj.camera_on_positive_y_axis());
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
