use std::sync::Mutex;

use cimvr_common::{
    nalgebra::{self, Isometry3, Point3, Vector3},
    Transform,
};
use cimvr_engine_interface::{make_app_state, prelude::*};

// Need a rand syscall because it's necessary in order to operate the ECS

struct State {
    head: EntityId,
}

make_app_state!(State);

impl AppState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        let head = io.create_entity();

        io.add_component(
            head,
            Transform {
                position: Point3::origin(),
                rotation: Isometry3::identity(),
                scale: Vector3::zeros(),
            },
        );

        schedule.add_system(Query, Self::system);

        Self { head }
    }
}

impl State {
    fn system(&mut self, io: &mut EngineIo) {
        todo!()
    }
}
