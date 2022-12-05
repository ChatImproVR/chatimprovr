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

static CTX: Lazy<Mutex<Context<State>>> = Lazy::new(|| Mutex::new(Context::new(State::new)));

// TODO: Put these behind a macro!

#[no_mangle]
fn reserve(bytes: u32) -> *mut u8 {
    CTX.lock().unwrap().reserve(bytes)
}

#[no_mangle]
fn dispatch() -> *mut u8 {
    CTX.lock().unwrap().dispatch()
}

impl State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        let head = io.create_entity();

        io.add_component(
            head,
            Transform {
                position: Point3::origin(),
                rotation: Isometry3::identity(),
                scale: Vector3::zeros(),
            },
        );

        schedule.add_system(Query, Self::system);

        Self { head }
    }

    fn system(&mut self, io: &mut EngineIo) {
        todo!()
    }
}
