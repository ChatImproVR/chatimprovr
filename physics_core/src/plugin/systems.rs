//! Systems responsible for interfacing our Cimvr components with the Rapier physics engine.

use super::{configuration::RapierConfiguration, context::RapierContext};

/// System responsible for advancing the physics simulation, and updating the internal state
/// for scene queries.
pub fn step_simulation<Hooks>(
    context: &mut RapierContext,
    config: RapierConfiguration,
    engine: &mut cimvr_engine::Engine,
    // hooks: StaticSystemParam<Hooks>,
    // time: Res<Time>,
    // mut sim_to_render_time: ResMut<SimulationToRenderTime>,
    // collision_events: EventWriter<CollisionEvent>,
    // contact_force_events: EventWriter<ContactForceEvent>,
    // interpolation_query: Query<(&RapierRigidBodyHandle, &mut TransformInterpolation)>,
) {
    // let context = context;
    // let hooks_adapter = BevyPhysicsHooksAdapter::new(hooks.into_inner());

    // Subscribe to FrameTime, CollisionEvent, ContactForceEvent
    // Query for RapierRigidBodyHandle, TransformInterpolation
    if config.physics_pipeline_active {
        // Advance the simulation one physics step.
        context.step_simulation(engine, config.gravity, config.timestep_mode);
        context.deleted_colliders.clear();
    } else {
        todo!("context.propagate_modified_body_positions_to_colliders();");
    }

    if config.query_pipeline_active {
        todo!("context.update_query_pipeline();");
    }
}
