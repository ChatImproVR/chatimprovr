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

            self.proj.update_proj(30., 60., &input);
        }

        let clear_color = [0.; 3];
        let new_projection = self.proj.matrices();

        for key in query.iter() {
            query.write::<Transform>(key, &self.proj.camera_on_positive_z_axis());
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
