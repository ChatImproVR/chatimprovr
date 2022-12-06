use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Isometry3, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// A transform component on an entity
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    //pub scale: Vector3<f32>,
}

impl Component for Transform {
    const ID: ComponentId = ComponentId {
        // steakhouse
        id: 0xDEAD_BEEF_CAFE,
        size: 30,
    };
}
