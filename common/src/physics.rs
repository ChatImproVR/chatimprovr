use crate::{make_handle, GenericHandle};
use cimvr_engine_interface::pkg_namespace;
use cimvr_engine_interface::prelude::*;
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub struct Physics {}

#[derive(Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RigidBodyType {
    #[default]
    Dynamic,
    Fixed,
    KinematicPositionBased,
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Default, PartialEq, Eq)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub handle: ColliderHandle,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ColliderShape {
    // Side Length
    Cube(f32),
    // Width, Hength, Length
    Prism(f32, f32, f32),
}

/// All information required to define a renderable mesh
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct LocalColliderMsg {
    /// Mesh data
    pub shape: ColliderShape,
    /// Unique ID
    pub handle: ColliderHandle,
}
/// Unique identifier for a RenderData resource
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ColliderHandle(GenericHandle);
make_handle!(ColliderHandle);

impl RigidBody {
    pub fn new(body_type: RigidBodyType, handle: ColliderHandle) -> Self {
        // TODO: Implement this where it registers the given rigidbody into the engine RigidBodySet
        RigidBody { body_type, handle }
    }

    pub fn add_force(&mut self, force_vec: Vec3, is_awake: bool) {
        todo!();
    }
}
