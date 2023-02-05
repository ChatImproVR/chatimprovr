use cimvr_common::{
    nalgebra::{self, Point2, Vector2},
    render::{Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pcg::Pcg, prelude::*, println};
mod sim;
use sim::*;
mod query_accel;

// All state associated with client-side behaviour
struct ClientState {
    sim: SimState,
    mesh: Mesh,
}

const SIM_RENDER_ID: RenderHandle = RenderHandle(0xBEEF_BEEF);

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut aa = Behaviour::default();

        // NOTE: We are using the println defined by cimvr_engine_interface here, NOT the standard library!
        let palette = SimConfig {
            colors: vec![[0.1, 1., 0.], [1., 0.1, 0.], [0., 0.25, 1.]],
            behaviours: vec![
                aa.with_inter_strength(10.),
                aa.with_inter_strength(2.),
                aa.with_inter_strength(-1.),
                aa.with_inter_strength(-1.),
                aa.with_inter_strength(10.),
                aa.with_inter_strength(2.),
                aa.with_inter_strength(2.),
                aa.with_inter_strength(-1.),
                aa.with_inter_strength(10.),
            ],
            damping: 5.,
        };

        let sim = SimState::new(&mut Pcg::new(), palette, 8_000);

        let ent = io.create_entity();
        io.add_component(ent, &Transform::identity());
        io.add_component(
            ent,
            &Render::new(SIM_RENDER_ID).primitive(Primitive::Points),
        );

        let mesh = Mesh::new();

        sched.add_system(Self::update, SystemDescriptor::new(Stage::Update));

        Self { sim, mesh }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let dt = 2e-5;
        self.sim.step(dt);

        let mesh = draw_particles(&self.sim);
        io.send(&RenderData {
            mesh,
            id: SIM_RENDER_ID,
        })
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        println!("Hello, server!");
        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);

fn draw_particles(sim: &SimState) -> Mesh {
    let mut vertices = vec![];
    let indices = (0..sim.particles().len() as u32).collect();
    for particle in sim.particles() {
        let color = sim.config().colors[particle.color as usize];

        let vertex = Vertex {
            pos: [particle.pos.x, 0., particle.pos.y],
            uvw: color,
        };

        vertices.push(vertex);
    }

    Mesh { vertices, indices }
}
