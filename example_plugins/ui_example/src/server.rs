use crate::ChangeColor;
use cimvr_common::{
    render::{Render, RenderExtra},
    ui::UiUpdate,
};
use cimvr_engine_interface::prelude::*;

pub struct ServerState;

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .query::<Render>(Access::Read)
            .subscribe::<UiUpdate>()
            .subscribe::<ChangeColor>()
            .build();

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(ChangeColor { rgb }) = io.inbox_first() {
            for entity in query.iter() {
                let mut extra = [0.; 4 * 4];
                extra[..3].copy_from_slice(&rgb);
                extra[3] = 1.;
                io.add_component(entity, RenderExtra(extra));
            }
        }
    }
}
