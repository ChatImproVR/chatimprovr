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
        // That's what I've been waitin for, that's what it's all about! Wahoo!
        id: 0x0000000_EEEAAA_BABEEE,
        locality: Locality::Local,
    };
}

impl AppState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create a new entity
        let head = io.create_entity();

        // Add the Transform component to it
        io.add_component(head, &Transform::default());

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule.add_system(
            SystemDescriptor {
                stage: Stage::Input,
                subscriptions: vec![sub::<StringMessage>()],
                query: vec![query::<Transform>(Access::Write)],
            },
            Self::my_system,
        );

        Self { head }
    }
}

impl State {
    fn my_system(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for StringMessage(txt) in io.inbox() {
            println!("Message: {}", txt);
        }

        for key in query.iter() {
            query.modify::<Transform>(key, |t| t.position.y += 0.1);

            let y = query.read::<Transform>(key).position.y;

            let txt = format!("Hewwo! {}", y);

            io.send(&StringMessage(txt));
        }
    }
}
