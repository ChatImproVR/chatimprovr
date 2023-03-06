use cimvr_common::ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState {
    ui: UiStateHelper,
    element: UiHandle,
}

struct ServerState;

make_app_state!(ClientState, ServerState);

/// Server to client chat message datatype
#[derive(Serialize, Deserialize, Debug)]
struct ChatDownload {
    username: String,
    message: String,
}

/// Client to server chat message datatype
#[derive(Serialize, Deserialize, Debug)]
struct ChatUpload(String);

const N_DISPLAYED_MESSAGES: usize = 5;

// Client code
impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched.add_system(
            Self::ui_update,
            SystemDescriptor::new(Stage::Update)
                .subscribe::<UiUpdate>()
                .subscribe::<ChatDownload>(),
        );

        let mut ui = UiStateHelper::new();

        // Create chat "window"
        let mut schema = vec![
            Schema::TextInput,
            Schema::Button {
                text: "Send".into(),
            },
        ];
        let mut state = vec![
            State::TextInput { text: "".into() },
            State::Button { clicked: false },
        ];
        for _ in 0..N_DISPLAYED_MESSAGES {
            schema.push(Schema::Label { text: "".into() });
            state.push(State::Label);
        }
        let element = ui.add(io, "Chat", schema, state);

        Self { ui, element }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        for msg in io.inbox::<ChatDownload>() {
            dbg!(msg);
        }

        if io.inbox::<UiUpdate>().next().is_some() {
            let ret = self.ui.read(self.element);
            let State::TextInput { text } = &ret[0] else { panic!() };

            if let State::Button { clicked: true } = ret[1] {
                io.send(&ChatUpload(text.to_string()));
            }
        }
    }
}

// Server code
impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Schedule the update() system to run every Update,
        // and allow it to receive the ChatMessage message
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).subscribe::<ChatUpload>(),
        );

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Dump both the message AND the client that sent the message to the console
        for (client, msg) in io.inbox_clients::<ChatUpload>() {
            dbg!((client, msg));
        }
    }
}

impl Message for ChatDownload {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("ChatDownload"),
        locality: Locality::Remote,
    };
}

impl Message for ChatUpload {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("ChatUpload"),
        locality: Locality::Remote,
    };
}
