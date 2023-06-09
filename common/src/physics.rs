use crate::{make_handle, GenericHandle};
use cimvr_engine_interface::pkg_namespace;
use cimvr_engine_interface::prelude::*;
use glam::Quat;
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub struct Physics {}

#[derive(Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum RigidBodyType {
    #[default]
    Dynamic,
    Fixed,
    KinematicPositionBased,
}

#[derive(Serialize, Deserialize, Copy, Clone, Default, PartialEq)]
pub struct RigidBodyBuilder {
    pub body_type: RigidBodyType,
    /// Bundles the collider with the rigidbody
    pub collider_handle: ColliderHandle,
    /// Glam vec
    translation: Vec3,
    rotation: f32,
    // position: (Vec3, Quat),
}

#[derive(Component, Serialize, Deserialize, Debug, Copy, Clone, Default, PartialEq)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    /// Bundles the collider with the rigidbody
    pub collider_handle: ColliderHandle,
    /// Glam vec
    pub translation: Vec3,
    pub rotation: f32,
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

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct LocalRigidBodyMsg {
    // entity: EntityId,
    action: PhysicsAction,
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct PhysicsAction {
    entity: EntityId,
    action: Action,
}

/*
 * This enum is what's in charge of the various physics actions in the engine
 * later on this will most likely be changed in favor of a physics helper, but
 * this will work for now.
 */
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Action {
    /// This is to add force to an entity's RigidBody
    Force(Vec3),
    // ...
}
/// Unique identifier for a RenderData resource
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ColliderHandle(GenericHandle);
make_handle!(ColliderHandle);

impl PhysicsAction {
    pub fn new(entity: EntityId, phys_action: Action) -> Self {
        Self {
            entity,
            action: phys_action,
        }
    }
}
impl RigidBody {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl LocalRigidBodyMsg {
    pub fn new(phys_action: PhysicsAction) -> Self {
        LocalRigidBodyMsg {
            // entity,
            action: phys_action,
        }
    }
}

impl RigidBodyBuilder {
    pub fn new(body_type: RigidBodyType, collider_handle: ColliderHandle) -> Self {
        // TODO: Implement this where it registers the given rigidbody into the engine RigidBodySet
        RigidBodyBuilder {
            body_type,
            collider_handle,
            rotation: Default::default(),
            translation: Vec3::ZERO,
        }
    }

    pub fn rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn translation(&mut self, translation: Vec3) {
        self.translation = translation;
    }

    pub fn build(&mut self) -> RigidBody {
        let mut rb = RigidBody::new();
        rb.translation = self.translation;
        rb.rotation = self.rotation;
        rb.collider_handle = self.collider_handle;
        rb.body_type = self.body_type;

        rb
    }
}
