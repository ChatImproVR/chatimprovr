use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use cimvr_common::{
    nalgebra::{self, Isometry3, Point3, UnitQuaternion, Vector3},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*, print, println, Locality};

struct State {
    head: EntityId,
}

make_app_state!(State);

impl AppState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        let head = io.create_entity();

        io.add_component(
            head,
            &Transform {
                position: Point3::origin(),
                rotation: UnitQuaternion::identity(),
            },
        );

        schedule.add_system(
            SystemDescriptor {
                stage: Stage::Input,
                subscriptions: vec![],
                query: vec![QueryTerm::new::<Transform>(Access::Write)],
            },
            Self::system,
        );

        Self { head }
    }
}

impl State {
    fn system(&mut self, io: &mut EngineIo) {
        println!("System runs!");
    }
}
