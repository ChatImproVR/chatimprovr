use cimvr_engine_interface::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

use crate::render::{Mesh, Vertex};

#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Local")]
pub struct PointcloudPacket {
    /// A list of points which retains it's order with respect to the location within the sensor's FOV
    // This is a u8 array as an optimization
    points: Vec<u8>,
    /// Whether or not each point can be trusted
    mask: Vec<bool>,
}

impl PointcloudPacket {
    pub fn new(points: Vec<Vertex>, mask: Vec<bool>) -> Self {
        Self {
            points: bytemuck::cast_slice(points.as_slice()).to_vec(),
            mask,
        }
    }

    pub fn points(&self) -> &[Vertex] {
        bytemuck::cast_slice(self.points.as_slice())
    }

    /// All valid points within the mesh
    pub fn valid_points(&self) -> impl Iterator<Item = Vertex> + '_ {
        self.points()
            .iter()
            .zip(&self.mask)
            .filter_map(|(pt, mask)| mask.then(|| *pt))
    }

    /// A mesh representing this pointcloud, for display purposes
    pub fn mesh(&self) -> Mesh {
        let vertices: Vec<Vertex> = self.valid_points().collect();
        Mesh {
            indices: (0..vertices.len() as u32).collect(),
            vertices,
        }
    }
}
