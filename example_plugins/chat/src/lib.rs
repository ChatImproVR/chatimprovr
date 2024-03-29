use cimvr_common::{
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
    utils::client_tracker::{Action, ClientTracker},
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState {
    ui: UiStateHelper,
    chat_window: UiHandle,
    displayed_messages: Vec<String>,
}

struct ServerState {
    tracker: ClientTracker,
}

make_app_state!(ClientState, ServerState);

/// Server to client chat message datatype;
/// used to tell clients to display the given chat message
#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Remote")]
pub struct ChatDownload {
    pub username: String,
    pub text: String,
}

/// Client to server chat message datatype;
/// used to tell the server to broadcast this chat message to all other clients
/// The server will decide how to set the username of the corresponding ChatDownload
#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Remote")]
pub struct ChatUpload(pub String);

/// Number of chat log messages
const N_DISPLAYED_MESSAGES: usize = 5;

// Client code
impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::ui_update)
            .subscribe::<UiUpdate>()
            .subscribe::<ChatDownload>()
            .build();

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
            chat_window: element,
            displayed_messages: vec![],
        }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Update the UI helper's internal state
        self.ui.download(io);

        // Check for UI updates
        if io.inbox::<UiUpdate>().next().is_some() {
            // Read the text input
            let ui_state = self.ui.read(self.chat_window);
            let State::TextInput { text } = &ui_state[0] else { panic!() };

            if let State::Button { clicked: true } = ui_state[1] {
                // Send chat message to server
                io.send(&ChatUpload(text.to_string()));

                // Clear the text input
                self.ui.modify(io, self.chat_window, |states| {
                    states[0] = State::TextInput { text: "".into() }
                });
            }
        }

        // Read chat messages from server
        let mut needs_update = false;
        for msg in io.inbox::<ChatDownload>() {
            // Format them and add them to the UI
            let disp = format!("{}: {}", msg.username, msg.text);
            self.displayed_messages.push(disp);

            // Rolling chat log
            if self.displayed_messages.len() > N_DISPLAYED_MESSAGES {
                self.displayed_messages.rotate_left(1);
                self.displayed_messages.pop();
            }

            needs_update = true;
        }

        // Display the chat log
        if needs_update {
            self.ui.modify(io, self.chat_window, |state| {
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
        sched
            .add_system(Self::update)
            .subscribe::<ChatUpload>()
            .subscribe::<Connections>()
            .build();

        Self {
            tracker: ClientTracker::new(),
        }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Get the list of connected clients (and their usernames)
        let Some(conns) = io.inbox_first() else { return; };

        // Connection/Disconnection messages
        let callback = |conn: &Connection, action: Action| {
            io.send(&ChatDownload {
                username: "SERVER".into(),
                text: match action {
                    Action::Connected => format!("User {} connected.", conn.username),
                    Action::Disconnected => format!("User {} disconnected.", conn.username),
                },
            })
        };

        self.tracker.update(&conns, callback);

        // Collect uploaded messages from clients
        let msgs = io.inbox_clients::<ChatUpload>().collect::<Vec<_>>();
        for (sender_client_id, ChatUpload(msg)) in msgs {
            // Find the sender's username
            let sender = self.tracker.clients().find(|c| c.id == sender_client_id);

            if let Some(sender) = sender {
                // Create a packet to send to all clients
                let msg = ChatDownload {
                    username: sender.username.clone(),
                    text: msg.clone(),
                };

                // Distribute it
                io.send(&msg);
            }
        }
    }
}
