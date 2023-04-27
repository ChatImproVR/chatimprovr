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
            .query("Color changers",Query::new().intersect::<Render>(Access::Read))
            .subscribe::<UiUpdate>()
            .subscribe::<ChangeColor>()
            .build();

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(ChangeColor { rgb }) = io.inbox_first() {
            for ent in query.iter("Color changers") {
                // The default shader uses RenderExtra to set the color
                let mut extra = [0.; 4 * 4];
                extra[..3].copy_from_slice(&rgb);
                // This value must be 1 to get the color to show. See the default vertex shader!
                extra[3] = 1.;
                io.add_component(ent, RenderExtra(extra));
            }
        }
    }
}
