use cimvr_common::{
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pcg::Pcg, pkg_namespace, prelude::*, println};
mod sim;
use sim::*;
mod query_accel;

// All state associated with client-side behaviour
struct ClientState {
    sim: SimState,
    time: f32,
}

const SIM_RENDER_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Simulation"));

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut aa = Behaviour::default();
        aa.inter_threshold = 0.05;

        let mut rand = || io.random() as u64 as f32 / u64::MAX as f32;

        let n = 5;

        let colors: Vec<[f32; 3]> = (0..n).map(|_| hsv_to_rgb(rand() * 360., 1., 1.)).collect();
        let behaviours = (0..n * n)
            .map(|_| aa.with_inter_strength((rand() * 2. - 1.) * 15.))
            .collect();

        // NOTE: We are using the println defined by cimvr_engine_interface here, NOT the standard library!
        let palette = SimConfig {
            colors,
            behaviours,
            /*
            colors: vec![
                [0.1, 1., 0.],
                [1., 0.1, 0.],
                [102. / 256., 30. / 256., 131. / 256.],
            ],
            behaviours: vec![
                aa.with_inter_strength(3.),
                aa.with_inter_strength(-1.5),
                aa.with_inter_strength(1.),
                aa.with_inter_strength(2.),
                aa.with_inter_strength(1.),
                aa.with_inter_strength(1.),
                aa.with_inter_strength(50.),
                aa.with_inter_strength(50.),
                aa.with_inter_strength(-100.),
            ],
            */
            damping: 150.,
        };

        dbg!(&palette);

        let sim = SimState::new(&mut Pcg::new(), palette, 8_000);

        io.create_entity()
            .add_component(Transform::identity())
            .add_component(Render::new(SIM_RENDER_ID).primitive(Primitive::Points))
            .build();

        sched.add_system(Self::update).build();

        Self { sim, time: 0. }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let dt = 1e-3;
        self.sim.step(dt);

        let mesh = draw_particles(&self.sim, self.time);
        io.send(&UploadMesh {
            mesh,
            id: SIM_RENDER_ID,
        });

        self.time += dt;
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

fn draw_particles(sim: &SimState, _time: f32) -> Mesh {
    let mut vertices = vec![];
    let indices = (0..sim.particles().len() as u32).collect();

    for particle in sim.particles().iter() {
        let color = sim.config().colors[particle.color as usize];

        let vertex = Vertex {
            pos: [particle.pos.x, 0., particle.pos.y],
            uvw: color,
        };

        vertices.push(vertex);
    }

    Mesh { vertices, indices }
}

/// https://gist.github.com/fairlight1337/4935ae72bcbcc1ba5c72
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s; // Chroma
    let h_prime = (h / 60.0) % 6.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (mut r, mut g, mut b);

    if 0. <= h_prime && h_prime < 1. {
        r = c;
        g = x;
        b = 0.0;
    } else if 1.0 <= h_prime && h_prime < 2.0 {
        r = x;
        g = c;
        b = 0.0;
    } else if 2.0 <= h_prime && h_prime < 3.0 {
        r = 0.0;
        g = c;
        b = x;
    } else if 3.0 <= h_prime && h_prime < 4.0 {
        r = 0.0;
        g = x;
        b = c;
    } else if 4.0 <= h_prime && h_prime < 5.0 {
        r = x;
        g = 0.0;
        b = c;
    } else if 5.0 <= h_prime && h_prime < 6.0 {
        r = c;
        g = 0.0;
        b = x;
    } else {
        r = 0.0;
        g = 0.0;
        b = 0.0;
    }

    r += m;
    g += m;
    b += m;

    [r, g, b]
}
