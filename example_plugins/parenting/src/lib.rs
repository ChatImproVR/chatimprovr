use cimvr_common::Transform;
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println};
use serde::{Deserialize, Serialize};

struct ClientState;
struct ServerState;

make_app_state!(ClientState, ServerState);

/// Sets our Transform to that of the given Entity,
/// prepending it with the Transform in this Component.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Parent(EntityId, Transform);

fn add_systems<State>(sched: &mut EngineSchedule<State>, f: Callback<State>) {
    sched
        .add_system(f)
        .query(
            "All Transforms",
            Query::new().intersect::<Transform>(Access::Read),
        )
        .query(
            "Objects With Parents",
            Query::new()
                .intersect::<Transform>(Access::Write)
                .intersect::<Parent>(Access::Read),
        )
        .build();
}

fn handle_parenting(query: &mut QueryResult) {
    // TODO: Determine the dependency graph for real; avoid loops!
    for child_id in query.iter("Objects With Parents") {
        let Parent(parent_id, prepend_tf) = query.read(child_id);
        if parent_id == child_id {
            println!(
                "Entity {:?} is parented to itself! Alabama would be proud.",
                parent_id
            );
            continue;
        }

        if query.has_component::<Transform>(parent_id) {
            let parent_tf = query.read(parent_id);
            query.write(child_id, &(prepend_tf * parent_tf));
        }
    }
}

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        add_systems(sched, Self::update);
        Self
    }
}

impl ServerState {
    fn update(&mut self, _io: &mut EngineIo, query: &mut QueryResult) {
        handle_parenting(query)
    }
}

impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        add_systems(sched, Self::update);
        Self
    }
}

impl ClientState {
    fn update(&mut self, _io: &mut EngineIo, query: &mut QueryResult) {
        handle_parenting(query)
    }
}

impl Default for Parent {
    fn default() -> Self {
        Self(
            EntityId(0xBAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD),
            Transform::identity(),
        )
    }
}
