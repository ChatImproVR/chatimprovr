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
    fn new(cmd: &mut EcsCommandBuf, schedule: &mut EngineSchedule<Self>) -> Self {
        let head = cmd.create_entity();

        cmd.add_component(
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
    fn system(&mut self, cmd: &mut EcsCommandBuf, query: &mut QueryTransaction) {
        for mut row in query.iter_mut() {
            cmd.add_component(row.entity(), &Transform::default());

            row.modify::<Transform>(|t| t.position.y += 0.1);
            row.modify::<Transform>(|t| {
                t.rotation *= UnitQuaternion::from_euler_angles(0.1, 0., 0.)
            });
        }
    }
}
