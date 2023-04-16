use std::collections::HashSet;

use cimvr_engine_interface::prelude::{Connection, Connections};

/// Runs a callback whenever a client connects or disconnects
#[derive(Default, Clone, Debug)]
pub struct ClientTracker(HashSet<Connection>);

impl ClientTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the client tracker, invoking the connect/disconnect callbacks as appropriate
    pub fn update(&mut self, conns: &Connections, mut callback: impl FnMut(&Connection, Action)) {
        let new_state: HashSet<Connection> = conns.clients.iter().cloned().collect();
        let ClientTracker(current_state) = self;

        for conn in current_state.difference(&new_state) {
            callback(conn, Action::Disconnected);
        }

        for conn in new_state.difference(&current_state) {
            callback(conn, Action::Connected);
        }

        *current_state = new_state;
    }

    /// Get the current roster of clients
    pub fn clients(&self) -> impl Iterator<Item = &Connection> {
        self.0.iter()
    }
}

/// Whether a client connected or disconnected
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Connected,
    Disconnected,
}
