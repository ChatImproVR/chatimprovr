use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion},
    Transform,
};
use cimvr_engine_interface::{
    dbg, make_app_state, prelude::*, print, println, serial::serialize, Locality,
};
use serde::{Deserialize, Serialize};

struct State {
    head: EntityId,
}

make_app_state!(State);

#[derive(Serialize, Deserialize)]
struct StringMessage(String);

impl Message for StringMessage {
    const CHANNEL: ChannelId = ChannelId {
        // That's what I've been wating for, that's what it's all about. Wahoo!
        id: 0xEEEAAA_BABEEE,
        locality: Locality::Local,
    };
}

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
                subscriptions: vec![StringMessage::CHANNEL],
                query: vec![query::<Transform>(Access::Write)],
            },
            Self::system,
        );

        Self { head }
    }
}

impl State {
    fn system(&mut self, cmd: &mut NonQueryIo, query: &mut QueryResult) {
        for StringMessage(txt) in cmd.inbox() {
            println!("Message: {}", txt);
        }

        for key in query.iter() {
            query.modify::<Transform>(key, |t| t.position.y += 0.1);

            let y = query.read::<Transform>(key).position.y;

            let txt = format!("Hewwo! {}", y);

            cmd.send(&StringMessage(txt));
        }
    }
}
