use cimvr_common::ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState {
    ui: UiStateHelper,
    element: UiHandle,
    displayed_messages: Vec<String>,
}

struct ServerState;

make_app_state!(ClientState, ServerState);

/// Server to client chat message datatype
#[derive(Serialize, Deserialize, Debug)]
struct ChatDownload {
    username: String,
    text: String,
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
            schema.push(Schema::Label);
            state.push(State::Label { text: "".into() });
        }
        let element = ui.add(io, "Chat", schema, state);

        Self {
            ui,
            element,
            displayed_messages: vec![],
        }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        if io.inbox::<UiUpdate>().next().is_some() {
            let ui_state = self.ui.read(self.element);
            let State::TextInput { text } = &ui_state[0] else { panic!() };

            if let State::Button { clicked: true } = ui_state[1] {
                io.send(&ChatUpload(text.to_string()));
            }
        }

        let mut needs_update = false;
        for msg in io.inbox::<ChatDownload>() {
            let disp = format!("{}: {}", msg.username, msg.text);
            self.displayed_messages.push(disp);
            if self.displayed_messages.len() > N_DISPLAYED_MESSAGES {
                self.displayed_messages.rotate_left(1);
                self.displayed_messages.pop();
            }
            needs_update = true;
        }

        if needs_update {
            self.ui.modify(io, self.element, |state| {
                for (label, disp) in state[2..].iter_mut().zip(&self.displayed_messages) {
                    if let State::Label { text } = label {
                        *text = disp.clone();
                    }
                }
            })
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
            SystemDescriptor::new(Stage::Update)
                .subscribe::<ChatUpload>()
                .subscribe::<Connections>(),
        );

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let Some(Connections { clients }) = io.inbox_first() else { return; };

        // Dump both the message AND the client that sent the message to the console
        let msgs = io.inbox_clients::<ChatUpload>().collect::<Vec<_>>();
        for (sender_client_id, ChatUpload(msg)) in msgs {
            let sender = clients.iter().find(|c| c.id == sender_client_id);

            if let Some(sender) = sender {
                let msg = ChatDownload {
                    username: sender.username.clone(),
                    text: msg.clone(),
                };

                for client in &clients {
                    if client.id != sender_client_id {
                        io.send_to_client(&msg, client.id);
                    }
                }
            }
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
