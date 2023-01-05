use crate::ChangeColor;
use cimvr_common::{
    render::{Mesh, Render, RenderData, RenderExtra, RenderHandle, Vertex},
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*, println};
use serde::{Deserialize, Serialize};

pub struct ServerState {
    rgb: [f32; 3],
    cube: EntityId,
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let rgb = [1.; 3];

        let cube = io.create_entity();
        io.add_component(cube, &Transform::default());

        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update)
                .query::<Render>(Access::Read)
                .subscribe::<UiUpdate>()
                .subscribe::<ChangeColor>(),
        );

        Self { rgb, cube }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(ChangeColor { rgb }) = io.inbox_first() {
            dbg!(rgb);
            self.rgb = rgb;
        }

        for key in query.iter() {
            let mut extra = [0.; 4 * 4];
            extra[..3].copy_from_slice(&self.rgb);
            extra[3] = 1.;
            io.add_component(key.entity(), &RenderExtra(extra));
        }
    }
}
