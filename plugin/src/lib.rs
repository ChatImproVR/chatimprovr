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
    fn new(cmd: &mut NonQueryIo, schedule: &mut EngineSchedule<Self>) -> Self {
        let head = cmd.create_entity();

        cmd.add_component(
            head,
            &Transform {
                position: Point3::origin(),
                rotation: UnitQuaternion::identity(),
            },
        );

        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
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
    fn system(&mut self, cmd: &mut NonQueryIo, query: &mut QueryResult) {
        dbg!(std::f32::consts::PI);

        for key in query.iter() {
            query.modify::<Transform>(key, |t| t.position.y += 0.1);
            query.modify::<Transform>(key, |t| {
                t.rotation *= UnitQuaternion::from_euler_angles(0.1, 0., 0.)
            });

            dbg!(query.read::<Transform>(key));
        }
    }
}
