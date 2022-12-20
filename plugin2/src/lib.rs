use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    input::{
        ElementState, InputEvent, InputEvents, KeyCode, KeyboardEvent, ModifiersState, MouseButton,
        MouseEvent,
    },
    nalgebra::{Point3, UnitQuaternion, Vector3, Vector4},
    render::{CameraComponent, Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    FrameTime, Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*, print, println};

struct State {}

make_app_state!(State);

impl UserState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
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

        Self {}
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
