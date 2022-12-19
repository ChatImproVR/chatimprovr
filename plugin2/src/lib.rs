use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

struct State {}

make_app_state!(State);

impl UserState for State {
    fn new(_cmd: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        schedule.add_system(
            SystemDescriptor {
                stage: Stage::Input,
                subscriptions: vec![],
                query: vec![QueryComponent::new::<Transform>(Access::Write)],
            },
            Self::system,
        );

        Self {}
    }
}

impl State {
    fn system(&mut self, cmd: &mut EngineIo, query: &mut QueryResult) {
        for key in query.iter() {
            query.modify::<Transform>(key, |t| {
                t.orient *= UnitQuaternion::from_euler_angles(0.1, 0., 0.)
            });

            dbg!(query.read::<Transform>(key));
        }

        let ent = cmd.create_entity();
        cmd.add_component(
            ent,
            &Transform {
                pos: Point3::new(0.1, 0.5, 0.8),
                orient: UnitQuaternion::identity(),
            },
        );
    }
}
