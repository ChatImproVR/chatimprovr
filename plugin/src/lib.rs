use std::sync::Mutex;

use cimvr_common::{
    nalgebra::{self, Isometry3, Point3, Vector3},
    Transform,
};
use cimvr_engine_interface::prelude::*;

// Need a rand syscall because it's necessary in order to operate the ECS

struct State {
    head: EntityId,
}

static CTX: Lazy<Context<State>> = Lazy::new(|| Context::new(State::new));

// TODO: Put these behind a macro!

#[no_mangle]
fn reserve(bytes: u32) -> *mut u8 {
    CTX.reserve(bytes)
}

#[no_mangle]
fn dispatch() -> *mut u8 {
    CTX.dispatch()
}

impl State {
    fn new(ctx: &mut EngineIo) -> Self {
        let head = ctx.create_entity();
        ctx.add_component(
            head,
            Transform {
                position: Point3::origin(),
                rotation: Isometry3::identity(),
                scale: Vector3::zeros(),
            },
        );

        ctx.add_system(Self::system);

        Self { head }
    }

    fn system(&mut self, query: &QueryResult) {
        todo!()
    }
}
