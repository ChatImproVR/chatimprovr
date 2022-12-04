pub use nalgebra;
use nalgebra::{Isometry3, Point3, Vector3};

pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Isometry3<f32>,
    pub scale: Vector3<f32>,
}
