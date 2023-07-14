use cimvr_common::{
    pointcloud::PointcloudPacket,
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct ClientState;

make_app_state!(ClientState, DummyUserState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const POINTCLOUD_RDR: MeshHandle = MeshHandle::new(pkg_namespace!("Pointcloud"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update_pcld)
            .subscribe::<PointcloudPacket>()
            .build();

        /*
        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(POINTCLOUD_RDR).primitive(Primitive::Points))
            .build();
        */

        Self
    }
}

impl ClientState {
    fn update_pcld(&mut self, io: &mut EngineIo, _: &mut QueryResult) {
        if let Some(packet) = io.inbox::<PointcloudPacket>().last() {
            dbg!(&packet.points().len());
            io.send(&UploadMesh {
                id: POINTCLOUD_RDR,
                mesh: packet.mesh(),
            });
        }
    }
}
