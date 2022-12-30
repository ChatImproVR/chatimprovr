use cimvr_engine_interface::dbg;
use cimvr_engine_interface::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// TODO: Create a derive macro which generates Vec<Schema> and Vec<State>, and consumes Vec<State>
// to do two-way data bindings for data structures. This could be implemented on components!

/// Handle to a unique UI element
#[derive(Serialize, Deserialize, Hash, Copy, Clone, Debug, Eq, PartialEq)]
pub struct UiHandle(u128);

/// UI element schema
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Schema {
    Label { text: String },
    Button { text: String },
    TextInput,
    DragValue { min: Option<f32>, max: Option<f32> },
}

/// UI element state
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum State {
    Label,
    Button { clicked: bool },
    TextInput { text: String },
    DragValue { value: f32 },
}

/// UI update message sent from plugins
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UiUpdate {
    pub id: UiHandle,
    pub state: Vec<State>,
}

/// UI request message sent to plugins
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UiRequest {
    pub id: UiHandle,
    pub op: UiOperation,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UiOperation {
    /// Create a new UI element
    Create {
        /// Name shown in UI
        name: String,
        /// Interface format
        schema: Vec<Schema>,
        /// Initial state
        init_state: Vec<State>,
    },
    /// Update the element's state
    Update(Vec<State>),
    /// Delete the UI element
    Delete,
}

pub struct UiStateHelper {
    map: HashMap<UiHandle, Vec<State>>,
}

impl UiStateHelper {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        io: &mut EngineIo,
        name: &str,
        schema: Vec<Schema>,
        init_state: Vec<State>,
    ) -> UiHandle {
        let id = UiHandle(io.random());

        self.map.insert(id, init_state.clone());

        let op = UiOperation::Create {
            name: name.to_string(),
            schema,
            init_state,
        };

        io.send(&UiRequest { id, op });

        id
    }

    pub fn read(&self, id: UiHandle) -> &[State] {
        self.map
            .get(&id)
            .expect("Attempted to read invalid UI handle")
    }

    pub fn modify<F: FnMut(&mut [State])>(&mut self, io: &mut EngineIo, id: UiHandle, mut f: F) {
        let state = self
            .map
            .get_mut(&id)
            .expect("Attempted to modify invalid UI handle");

        let old_state = state.to_vec();
        f(state);

        assert!(
            old_state
                .iter()
                .zip(state.iter())
                .all(|(o, n)| std::mem::discriminant(o) == std::mem::discriminant(n)),
            "Cannot modify UI state datatypes"
        );

        let op = UiOperation::Update(state.to_vec());
        io.send(&UiRequest { id, op });
    }

    pub fn delete(&mut self, io: &mut EngineIo, id: UiHandle) {
        self.map
            .remove(&id)
            .expect("Attempted to delete invalid UI handle");

        let op = UiOperation::Delete;
        io.send(&UiRequest { id, op });
    }

    pub fn download(&mut self, io: &mut EngineIo) {
        for msg in io.inbox::<UiUpdate>() {
            if let Some(state) = self.map.get_mut(&msg.id) {
                *state = msg.state;
            }
        }
    }
}

impl Message for UiRequest {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x239840283048,
        locality: Locality::Local,
    };
}

impl Message for UiUpdate {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x3284080238423,
        locality: Locality::Local,
    };
}
