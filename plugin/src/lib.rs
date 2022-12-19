use cimvr_common::{
    input::InputEvents,
    render::{Primitive, Render, RenderHandle},
    StringMessage, Transform,
};
use cimvr_engine_interface::{make_app_state, prelude::*, println};

struct State {
    head: EntityId,
}

make_app_state!(State);

impl UserState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create a new entity
        let head = io.create_entity();

        // Add the Transform component to it
        io.add_component(head, &Transform::default());
        io.add_component(
            head,
            &Render {
                id: RenderHandle(3984203840),
                primitive: Primitive::Lines,
                limit: 0,
            },
        );

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule.add_system(
            SystemDescriptor {
                stage: Stage::Input,
                subscriptions: vec![sub::<StringMessage>(), sub::<InputEvents>()],
                query: vec![query::<Transform>(Access::Write)],
            },
            Self::my_system,
        );

        Self { head }
    }
}

impl State {
    fn my_system(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Receive messages
        for StringMessage(txt) in io.inbox() {
            println!("String message: {}", txt);
        }

        // Receive messages
        for InputEvents(txt) in io.inbox() {
            println!("Input events: {:#?}", txt);
        }

        // Iterate through the query
        for key in query.iter() {
            query.modify::<Transform>(key, |t| t.pos.y += 0.1);

            let y = query.read::<Transform>(key).pos.y;

            if key.entity() == self.head {
                let txt = format!("Head y pos: {}", y);
                io.send(&StringMessage(txt));
            }
        }
    }
}
