use ahash::HashMap;
use cimvr_engine_interface::prelude::EntityId;
// use rapier;
use rapier3d::prelude::*;
// use rapier:
/// The Rapier context, containing all the state of the physics engine.
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
// #[derive(Resource)]
pub struct RapierContext {
    /// The island manager, which detects what object is sleeping
    /// (not moving much) to reduce computations.
    pub islands: IslandManager,
    /// The broad-phase, which detects potential contact pairs.
    pub broad_phase: BroadPhase,
    /// The narrow-phase, which computes contact points, tests intersections,
    /// and maintain the contact and intersection graphs.
    pub narrow_phase: NarrowPhase,
    /// The set of rigid-bodies part of the simulation.
    pub bodies: RigidBodySet,
    /// The set of colliders part of the simulation.
    pub colliders: ColliderSet,
    /// The set of impulse joints part of the simulation.
    pub impulse_joints: ImpulseJointSet,
    /// The set of multibody joints part of the simulation.
    pub multibody_joints: MultibodyJointSet,
    /// The solver, which handles Continuous Collision Detection (CCD).
    pub ccd_solver: CCDSolver,
    /// The physics pipeline, which advance the simulation step by step.
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub pipeline: PhysicsPipeline,
    /// The query pipeline, which performs scene queries (ray-casting, point projection, etc.)
    pub query_pipeline: QueryPipeline,
    /// The integration parameters, controlling various low-level coefficient of the simulation.
    pub integration_parameters: IntegrationParameters,
    pub(crate) physics_scale: Real,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) event_handler: Option<Box<dyn EventHandler>>,
    // For transform change detection.
    // #[cfg_attr(feature = "serde-serialize", serde(skip))]
    // pub(crate) last_body_transform_set: HashMap<RigidBodyHandle, GlobalTransform>,
    // NOTE: these maps are needed to handle despawning.
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) entity2body: HashMap<EntityId, RigidBodyHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) entity2collider: HashMap<EntityId, ColliderHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) entity2impulse_joint: HashMap<EntityId, ImpulseJointHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) entity2multibody_joint: HashMap<EntityId, MultibodyJointHandle>,
    // This maps the handles of colliders that have been deleted since the last
    // physics update, to the entity they was attached to.
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) deleted_colliders: HashMap<ColliderHandle, EntityId>,
    // NOTE: these maps are needed to handle despawning.
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) character_collisions_collector: Vec<rapier3d::control::CharacterCollision>,
}

fn stuff() {
    println!("rawr");
}
