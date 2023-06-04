//! Types used for communication with the engine
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Read, Write},
};

use crate::prelude::*;
use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Fixed-size Option type, for use within components
/// Note that this type implements From/Into for Option<T>
#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub struct FixedOption<T>(bool, T);

/// Serialize the message in the standard binary format
pub fn serialize<T: Serialize>(val: &T) -> bincode::Result<Vec<u8>> {
    bincode_opts().serialize(val)
}

/// Serialize the message in the standard binary format
pub fn serialize_into<W: Write, T: Serialize>(w: W, val: &T) -> bincode::Result<()> {
    bincode_opts().serialize_into(w, val)
}

/// Get the serialized size of the message.
pub fn serialized_size<T: Serialize>(val: &T) -> bincode::Result<usize> {
    Ok(bincode_opts().serialized_size(val)? as usize)
}

/// Deserialize the message in the standard binary format
pub fn deserialize<R: Read, T: DeserializeOwned>(r: R) -> bincode::Result<T> {
    bincode_opts().deserialize_from(r)
}

// (TODO: Make this just a header containing references to a dense buffer
/// Plugin-local ECS data
/// Represents the data returned by a query
pub type EcsData = HashMap<ComponentId, HashMap<EntityId, Vec<u8>>>;

/// Data transferred from Host to Plugin
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReceiveBuf {
    /// Which system to execute, None if initializing
    pub system: Option<usize>,
    /// Compact ECS data
    pub ecs: EcsData,
    /// Message inbox
    pub inbox: Inbox,
    /// True if plugin is server-side
    pub is_server: bool,
}

/// Data transferred from Plugin to Host
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SendBuf {
    /// Commands to be sent to server
    pub commands: Vec<EcsCommand>,
    /*
    /// Messages to be sent to other plugins
    pub messages: Vec<Message>,
    */
    /// Schedule setup on init. Must be empty except for first use!
    pub systems: Vec<SystemDescriptor>,
    /// Message outbox
    pub outbox: Vec<MessageData>,
}

fn bincode_opts() -> impl Options {
    // NOTE: This is actually different from the default bincode serialize() function!!
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
}

impl<T: Default> Default for FixedOption<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> FixedOption<T> {
    /// Create a new Some variant
    pub fn some(t: T) -> Self {
        Self(true, t)
    }

    /// Create a new None variant
    pub fn none() -> Self
    where
        T: Default,
    {
        Self(false, T::default())
    }

    pub fn is_some(&self) -> bool {
        self.0
    }

    pub fn is_none(&self) -> bool {
        !self.0
    }

    pub fn as_ref(&self) -> FixedOption<&T> {
        FixedOption(self.0, &self.1)
    }
}

impl<T> Into<Option<T>> for FixedOption<T> {
    fn into(self) -> Option<T> {
        let Self(b, t) = self;
        b.then(|| t)
    }
}

impl<T: Default> From<Option<T>> for FixedOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Option::None => Self::none(),
            Some(t) => Self::some(t),
        }
    }
}

impl<T: Default + Debug> Debug for FixedOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opt: Option<&T> = self.as_ref().into();
        opt.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedOption;
    use crate::is_fixed_size;

    #[test]
    fn ser_fixed_option() {
        let a = FixedOption::some(8u32);
        is_fixed_size(a).unwrap();
    }

    #[test]
    #[should_panic]
    fn ser_fixed_option_bad() {
        let a = FixedOption::some(vec![8u32]);
        is_fixed_size(a).unwrap();
    }
}
