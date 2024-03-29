use cimvr_common::{
    desktop::InputEvent, render::CameraComponent, utils::camera::Orthographic, Transform,
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
            .query(
                "Camera",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<CameraComponent>(Access::Write),
            )
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

        for entity in query.iter("Camera") {
            query.write::<Transform>(entity, &self.proj.camera_on_positive_z_axis());
        }

        for entity in query.iter("Camera") {
            query.write::<CameraComponent>(
                entity,
                &CameraComponent {
                    clear_color,
                    projection: new_projection,
                },
            );
        }
    }
}
