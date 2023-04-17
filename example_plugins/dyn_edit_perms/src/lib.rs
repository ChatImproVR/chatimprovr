use cimvr_engine_interface::{
    dbg,
    dyn_edit::{DynamicEditCommand, DynamicEditRequest},
    make_app_state, pkg_namespace,
    prelude::*,
    println, ComponentSchema,
};
use serde::{Deserialize, Serialize};

struct ServerState;

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<DynamicEditRequest>()
            .subscribe::<ComponentSchema>()
            .build();
        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Automatically forward all edit requests into edit commands.
        // Here you might filter by username, permissions, etc.
        for DynamicEditRequest(edit) in io.inbox().collect::<Vec<_>>() {
            io.send(&DynamicEditCommand(edit));
        }

        // The second purpose of this plugin:
        // Receive component schema server-side, and forward them
        // client-side for display/editing
        for component_schema in io.inbox::<ComponentSchema>().collect::<Vec<_>>() {
            io.send(&ComponentSchemaDownload(component_schema));
        }
    }
}

#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Remote")]
struct ComponentSchemaDownload(ComponentSchema);

struct ClientState;

impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<ComponentSchemaDownload>()
            .build();
        Self
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Download component schema data from server and make it available client-side.
        for ComponentSchemaDownload(component_schema) in
            io.inbox::<ComponentSchemaDownload>().collect::<Vec<_>>()
        {
            io.send(&component_schema);
        }
    }
}

make_app_state!(ClientState, ServerState);
