use cimvr_engine_interface::{
    dyn_edit::{DynamicEditCommand, DynamicEditRequest},
    make_app_state,
    prelude::*,
};

struct ServerState;

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Automatically forward all edit requests into edit commands.
        // Here you might filter by username, permissions, etc.
        for DynamicEditRequest(edit) in io.inbox().collect::<Vec<_>>() {
            io.send(&DynamicEditCommand(edit));
        }
    }
}

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<DynamicEditRequest>()
            .build();
        Self
    }
}

make_app_state!(DummyUserState, ServerState);
