use cimvr_common::{
    glam::{Mat3, Quat, Vec3, Vec4},
    render::{CameraComponent},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

struct Camera2D;
make_app_state!(Camera2D, DummyUserState);


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

        Self
    }
}

impl Camera2D {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        
    }
}