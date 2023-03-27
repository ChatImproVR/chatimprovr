#[allow(unused)]
use std::ops::Deref;

use super::{configuration::RapierConfiguration, context::RapierContext};
use anyhow::Result;
use cimvr_engine::Engine;
use cimvr_engine_interface::{pkg_namespace, prelude::*, FrameTime};
use rapier::prelude::*;
struct PhysicsPlugin {
    rapier_context: RapierContext,
    rapier_configuration: RapierConfiguration,
}

#[derive(Message, Serialize, Deserialize)]
#[locality("Local")]
pub struct CimvrCollisionEvent(CollisionEvent);

// pub struct CimvrContactForceEvent(ContactForceEvent);

#[derive(Message, Serialize, Deserialize)]
#[locality("Local")]
pub struct CimvrContactForceEvent {
    /// The first collider involved in the contact.
    pub collider1: ColliderHandle,
    /// The second collider involved in the contact.
    pub collider2: ColliderHandle,
    /// The sum of all the forces between the two colliders.
    pub total_force: Vector<Real>,
    /// The sum of the magnitudes of each force between the two colliders.
    ///
    /// Note that this is **not** the same as the magnitude of `self.total_force`.
    /// Here we are summing the magnitude of all the forces, instead of taking
    /// the magnitude of their sum.
    pub total_force_magnitude: Real,
    /// The world-space (unit) direction of the force with strongest magnitude.
    pub max_force_direction: Vector<Real>,
    /// The magnitude of the largest force at a contact point of this contact pair.
    pub max_force_magnitude: Real,
}

impl From<CollisionEvent> for CimvrCollisionEvent {
    fn from(value: CollisionEvent) -> Self {
        CimvrCollisionEvent(value)
    }
}

impl From<ContactForceEvent> for CimvrContactForceEvent {
    fn from(value: ContactForceEvent) -> Self {
        Self {
            collider1: value.collider1,
            collider2: value.collider2,
            total_force: value.total_force,
            total_force_magnitude: value.total_force_magnitude,
            max_force_direction: value.max_force_direction,
            max_force_magnitude: value.max_force_magnitude,
        }
    }
}

impl PhysicsPlugin {
    pub fn new(engine: &mut Engine) -> Result<Self> {
        engine.subscribe::<FrameTime>();
        engine.subscribe::<CimvrCollisionEvent>();
        let rapier_context = RapierContext::default();
        let rapier_configuration = RapierConfiguration::default();

        Ok(Self {
            rapier_context,
            rapier_configuration,
        })
    }

    /// System responsible for advancing the physics simulation, and updating the internal state
    /// for scene queries.
    pub fn step_simulation(&mut self, engine: &mut Engine) {
        if self.rapier_configuration.physics_pipeline_active {
            // Advance the simulation one physics step.

            self.rapier_context.step_simulation(
                engine,
                self.rapier_configuration.gravity,
                self.rapier_configuration.timestep_mode,
            );
            self.rapier_context.deleted_colliders.clear();
        } else {
            todo!("self.rapier_context.propagate_modified_body_positions_to_colliders();");
        }

        if self.rapier_configuration.query_pipeline_active {
            todo!("self.rapier_context.update_query_pipeline();");
        }
    }

    pub fn apply_scale() {
        // config: Res<RapierConfiguration>,
        //     mut changed_collider_scales: Query<
        //         (&mut Collider, &GlobalTransform, Option<&ColliderScale>),
        //         Or<(
        //             Changed<Collider>,
        //             Changed<GlobalTransform>,
        //             Changed<ColliderScale>,
        //         )>,
        //     >,
        todo!("Need to implement apply_scale function");
    }
}
