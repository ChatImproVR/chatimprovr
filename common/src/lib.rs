#![cfg(not(doctest))]
//! # Common
//! This crate is intended to facilitate communication with the specific server and client
//! implementations provided alongside ChatimproVR. This library is always used in conjunction with
//! `engine_interface`.

use cimvr_engine_interface::pkg_namespace;
use cimvr_engine_interface::prelude::*;
pub use glam;
use serde::{Deserialize, Serialize};

pub mod desktop;
pub mod gamepad;
mod generic_handle;
pub mod pointcloud;
pub mod render;
mod transform;
pub mod ui;
pub mod utils;
pub mod vr;

pub use generic_handle::GenericHandle;
pub use transform::Transform;

/// Requests that the client disconnect from the current server in favor of this new server
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[locality("Local")]
pub struct InterdimensionalTravelRequest {
    pub address: String,
}
