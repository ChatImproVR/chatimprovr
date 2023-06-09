use std::collections::{HashMap, HashSet};

use cimvr_common::physics::{RigidBody, RigidBodyBuilder};
use cimvr_engine::interface::prelude::EntityId;
use rapier3d::prelude::Collider;
/*
 * This PhysicsManager is composed of some builders that act as a middleman from
 * the plugins to the rapier engine which then modifies the transforms directly.
 *
 * The plugins will send local messages server side with potential changes to colliders
 * and rigidbodies that the manager will then pass that information to rapier's island manager,
 * and make the changes to the transforms and other relevant data on the entities' components.
 */
struct PhysicsManager {
    cimvr_rigidbody_set: HashMap<EntityId, RigidBody>,
    cimvr_collider_set: HashMap<EntityId, Collider>,
}

impl PhysicsManager {}
