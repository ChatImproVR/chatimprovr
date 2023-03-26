use crate::prelude::*;
use cimvr_engine::Engine;
use rapier::prelude::*;

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
    #[cfg(feature = "joints")]
    pub impulse_joints: ImpulseJointSet,
    /// The set of multibody joints part of the simulation.
    #[cfg(feature = "joints")]
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
    pub(crate) entity2body: HashMap<Entity, RigidBodyHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) entity2collider: HashMap<Entity, ColliderHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    #[cfg(feature = "joints")]
    pub(crate) entity2impulse_joint: HashMap<Entity, ImpulseJointHandle>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    #[cfg(feature = "joints")]
    pub(crate) entity2multibody_joint: HashMap<Entity, MultibodyJointHandle>,
    // This maps the handles of colliders that have been deleted since the last
    // physics update, to the entity they was attached to.
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) deleted_colliders: HashMap<ColliderHandle, Entity>,
    #[cfg_attr(feature = "serde-serialize", serde(skip))]
    pub(crate) character_collisions_collector: Vec<rapier::control::CharacterCollision>,
}

impl RapierContext {
    pub fn step_simulation(&mut self, engine: &mut Engine) {
        //     pub fn step_simulation(
        //     &mut self,
        //     gravity: Vect,
        //     timestep_mode: TimestepMode,
        //     events: Option<(EventWriter<CollisionEvent>, EventWriter<ContactForceEvent>)>,
        //     hooks: &dyn PhysicsHooks,
        //     time: &Time,
        //     sim_to_render_time: &mut SimulationToRenderTime,
        //     mut interpolation_query: Option<
        //         Query<(&RapierRigidBodyHandle, &mut TransformInterpolation)>,
        //     >,
        // )
        let ecs = engine.ecs();
        // let time = engine.dispatch_plugin
        // ecs.query()
    }
}

impl Default for RapierContext {
    fn default() -> Self {
        Self {
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            #[cfg(feature = "joints")]
            impulse_joints: ImpulseJointSet::new(),
            #[cfg(feature = "joints")]
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            pipeline: PhysicsPipeline::new(),
            query_pipeline: QueryPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_scale: 1.0,
            event_handler: None,
            // last_body_transform_set: HashMap::new(),
            entity2body: HashMap::new(),
            entity2collider: HashMap::new(),
            #[cfg(feature = "joints")]
            entity2impulse_joint: HashMap::new(),
            #[cfg(feature = "joints")]
            entity2multibody_joint: HashMap::new(),
            deleted_colliders: HashMap::new(),
            character_collisions_collector: vec![],
        }
    }
}
