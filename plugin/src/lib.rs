use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion},
    Transform,
};
use cimvr_engine_interface::{
    dbg, make_app_state, prelude::*, print, println, serial::serialize, Locality,
};

struct State {
    head: EntityId,
}

make_app_state!(State);

const TEST_CHAN: ChannelId = ChannelId {
    id: 28308423098094823,
    locality: Locality::Local,
};

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
                subscriptions: vec![TEST_CHAN],
                query: vec![QueryTerm::new::<Transform>(Access::Write)],
            },
            Self::system,
        );

        Self { head }
    }
}

impl State {
    fn system(&mut self, cmd: &mut NonQueryIo, query: &mut QueryResult) {
        println!("HEWWO?");
        if let Some(inbox) = &cmd.inbox().get(&TEST_CHAN) {
            for msg in *inbox {
                println!(
                    "{:?}: {}",
                    msg.channel,
                    String::from_utf8(msg.data.clone()).unwrap()
                );
            }
        } else {
            println!("Empty inbox qwq");
        }

        for key in query.iter() {
            query.modify::<Transform>(key, |t| t.position.y += 0.1);
            let k = query.read::<Transform>(key).position.y;
            cmd.send(TEST_CHAN, format!("Hewwo! {}", k).bytes().collect());
        }
    }
}
