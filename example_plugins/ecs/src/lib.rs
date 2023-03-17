use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState;

struct ServerState {
    increment: i32,
}

make_app_state!(ClientState, ServerState);

/// Component datatype
/// Implements Serialize and Deserialize, making it compatible with the Component trait.
#[derive(Component, Serialize, Deserialize, Default, Clone, Copy, Debug)]
struct MyComponent {
    a: i32,
    b: f32,
}

// Server code
impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Create a new entity
        let ent = io.create_entity();
        // Add MyComponent to it, so that it's updated in update()
        io.add_component(ent, &MyComponent { a: 0, b: 0.0 });
        // Add Sychronized to it, so that it is sent to the client each frame
        io.add_component(ent, &Synchronized);

        // Schedule the update() system to run every Update
        // Queries all entities with MyComponent attached, and allows us to write to them
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).query::<MyComponent>(Access::Write),
        );

        Self { increment: 0 }
    }
}

impl ServerState {
    fn update(&mut self, _io: &mut EngineIo, query: &mut QueryResult) {
        // Update MyComponent, which is then automatically Sychronized with the connected clients
        for key in query.iter() {
            query.write(
                key,
                &MyComponent {
                    a: self.increment,
                    b: self.increment as f32,
                },
            );
        }

        self.increment += 1;
    }
}

// Client code
impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Schedule the update() system to run every Update,
        // querying all entities with the MyComponent component attached
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).query::<MyComponent>(Access::Read),
        );

        Self
    }
}

impl ClientState {
    fn update(&mut self, _io: &mut EngineIo, query: &mut QueryResult) {
        // Write all MyComponents to the console
        for key in query.iter() {
            dbg!(query.read::<MyComponent>(key));
        }
    }
}
