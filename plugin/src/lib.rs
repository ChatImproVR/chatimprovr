use cimvr_common::{
    input::InputEvents,
    nalgebra::{Point3, UnitQuaternion, Vector3},
    render::{CameraComponent, Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    FrameTime, Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*, print, println};

struct State {
    head: EntityId,
}

make_app_state!(State);

impl UserState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create head
        let head = io.create_entity();
        let camera_pos = Point3::new(3., 3., 3.);
        io.add_component(
            head,
            &Transform {
                pos: camera_pos,
                orient: UnitQuaternion::face_towards(&camera_pos.coords, &Vector3::y()),
            },
        );
        io.add_component(head, &CameraComponent);

        // Craate cube
        let cube_ent = io.create_entity();
        let cube_mesh = cube();
        io.add_component(cube_ent, &Transform::default());
        io.add_component(
            cube_ent,
            &Render {
                id: cube_mesh.id,
                primitive: Primitive::Triangles,
                limit: None,
            },
        );

        io.send(&cube_mesh);

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule.add_system(
            SystemDescriptor {
                stage: Stage::Input,
                subscriptions: vec![sub::<FrameTime>(), sub::<InputEvents>()],
                query: vec![
                    query::<Transform>(Access::Write),
                    query::<CameraComponent>(Access::Read),
                ],
            },
            Self::camera_move,
        );

        Self { head }
    }
}

fn cube() -> RenderData {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    RenderData {
        mesh: Mesh { vertices, indices },
        id: RenderHandle(3984203840),
    }
}

impl State {
    fn camera_move(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Receive messages

        // Receive messages
        for InputEvents(txt) in io.inbox() {
            println!("Input events: {:#?}", txt);
        }

        // Iterate through the query
        if let Some(time) = io.inbox_first::<FrameTime>() {
            for key in query.iter() {
                query.modify::<Transform>(key, |t| t.pos.y = time.time * 0.3);

                /*
                let y = query.read::<Transform>(key).pos.y;

                if key.entity() == self.head {
                let txt = format!("Head y pos: {}", y);
                io.send(&StringMessage(txt));
                }
                */
            }
        }
    }
}
