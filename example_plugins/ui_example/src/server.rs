use crate::ChangeColor;
use cimvr_common::{
    render::{Render, RenderExtra},
    ui::UiUpdate,
    Transform,
};
use cimvr_engine_interface::prelude::*;

pub struct ServerState {
    cube: EntityId,
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let cube = io
            .create_entity()
            .add_component(Transform::default())
            .build();

        sched
            .add_system(Self::update)
            .query::<Render>(Access::Read)
            .subscribe::<UiUpdate>()
            .subscribe::<ChangeColor>()
            .build();

        Self { cube }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        if let Some(ChangeColor { rgb }) = io.inbox_first() {
            let mut extra = [0.; 4 * 4];
            extra[..3].copy_from_slice(&rgb);
            extra[3] = 1.;
            io.add_component(self.cube, RenderExtra(extra));
        }
    }
}
