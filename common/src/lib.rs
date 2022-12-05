use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Isometry3, Point3, Vector3};
use serde::{Deserialize, Serialize};

/// A transform component on an entity
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Isometry3<f32>,
    pub scale: Vector3<f32>,
}

impl Component for Transform {
    const ID: ComponentId = ComponentId {
        id: 0xDEAD_BEEF_CAFE,
        size: 28,
    };
}
