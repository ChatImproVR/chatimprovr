use cimvr_common::{
    render::CameraComponent,
    utils::camera::Orthographic,
    Transform,
    desktop::{InputEvent},
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

struct Camera2D {
    proj: Orthographic,
}
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
        schedule
            .add_system(Self::update)
            .stage(Stage::PreUpdate)
            .subscribe::<InputEvent>()
            .query("Camera")
                .intersect::<Transform>(Access::Write)
                .intersect::<CameraComponent>(Access::Write)
                .qcommit()
            .build();

        Self {
            proj: Orthographic::new(),
        }
    }
}

impl Camera2D {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        
        for input in io.inbox::<InputEvent>() {

            // Provie the width and the height of the world application size
            self.proj.update_proj(80., 120., &input);
        }

        let clear_color = [0.; 3];
        let new_projection = self.proj.matrices();

        for key in query.iter("Camera") {
            query.write::<Transform>(key, &self.proj.camera_on_positive_z_axis());
        }
        

        for key in query.iter("Camera") {
            query.write::<CameraComponent>(
                key,
                &CameraComponent {
                    clear_color,
                    projection: new_projection,
                },
            );
        }
    }
}
