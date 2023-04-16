use serde::{Deserialize, Serialize};

/// A generic handle type, which is integer sized but represents a namespace
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GenericHandle(u128);

impl GenericHandle {
    /// Create a handle from the given name. Hashes the string
    pub const fn new(name: &str) -> Self {
        Self(const_hash(name))
    }

    /// Create a handle within this namespace, indexed by `i`.
    /// Note that this is a deterministic function!
    pub fn index(self, i: u128) -> Self {
        Self(self.0.wrapping_add(i))
    }
}

/// A pretty bad hash function. Made constant so you can have things like
/// ```rust
/// use cimvr_engine_interface::{pkg_namespace, prelude::*};
/// use cimvr_common::render::RenderHandle;
/// const CUBE_HANDLE: RenderHandle = RenderHandle::new(pkg_namespace!("Cube"));
/// ```
const fn const_hash(s: &str) -> u128 {
    const C: u128 = 31;
    let mut hash: u128 = 0;
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        let b = bytes[i] as u128;
        hash = hash.wrapping_mul(C).wrapping_add(b);
        i += 1;
    }
    hash
}

/// Creates a handle type wrapping a GenericHandle. For example:
/// ```rust
/// struct MyHandle(GenericHandle);
/// make_handle!(MyHandle);
/// ```
#[macro_export]
macro_rules! make_handle {
    ($name:ident) => {
        impl $name {
            /// Create a handle from the given name. Hashes the string
            pub const fn new(name: &str) -> Self {
                Self(GenericHandle::new(name))
            }

            /// Create a handle within this namespace, indexed by `i`.
            /// Note that this is a deterministic function!
            pub fn index(self, i: u128) -> Self {
                Self(self.0.index(i))
            }
        }
    };
}

impl Default for GenericHandle {
    fn default() -> Self {
        Self(0xBAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD)
    }
}
