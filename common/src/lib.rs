#![cfg(not(doctest))]
//! # Common
//! This crate is intended to facilitate communication with the specific server and client
//! implementations provided alongside ChatimproVR. This library is always used in conjunction with
//! `engine_interface`.

pub use glam;

pub mod desktop;
pub mod gamepad;
mod generic_handle;
pub mod render;
mod transform;
pub mod ui;
pub mod utils;
pub mod vr;

pub use generic_handle::GenericHandle;
pub use transform::Transform;
