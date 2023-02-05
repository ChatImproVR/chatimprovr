use cimvr_common::{
    nalgebra::{self, Point2, Vector2},
    render::{Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pcg::Pcg, prelude::*, println};

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
        let palette = Palette {
            colors: vec![[0.1, 1., 0.], [1., 0.1, 0.], [0., 0.25, 1.]],
            behaviours: vec![
                aa.with_inter_strength(10.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(10.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(0.),
                aa.with_inter_strength(10.),
            ],
        };

        let sim = SimState::new(&mut Pcg::new(), palette, 1_000);

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

struct SimState {
    particles: Vec<Particle>,
    palette: Palette,
}

type Color = u8;

#[derive(Clone, Copy)]
struct Particle {
    pos: Point2<f32>,
    vel: Vector2<f32>,
    color: Color,
}

#[derive(Clone, Copy)]
pub struct Behaviour {
    /// Magnitude of the default repulsion force
    pub default_repulse: f32,
    /// Zero point between default repulsion and particle interaction (0 to 1)
    pub inter_threshold: f32,
    /// Interaction peak strength
    pub inter_strength: f32,
    /// Maximum distance of particle interaction (0 to 1)
    pub inter_max_dist: f32,
}

/// Display colors and physical behaviour coefficients
struct Palette {
    colors: Vec<[f32; 3]>,
    behaviours: Vec<Behaviour>,
}

impl Behaviour {
    /// Returns the force on this particle
    ///
    /// Distance is in the range `0.0..=1.0`
    fn interact(&self, dist: f32) -> f32 {
        if dist < self.inter_threshold {
            let f = dist / self.inter_threshold;
            (1. - f) * -self.default_repulse
        } else if dist > self.inter_max_dist {
            0.0
        } else {
            let x = dist - self.inter_threshold;
            let x = x / (self.inter_max_dist - self.inter_threshold);
            let x = x * 2. - 1.;
            let x = 1. - x.abs();
            x * self.inter_strength
        }
    }
}

impl SimState {
    pub fn new(rng: &mut Pcg, palette: Palette, n: usize) -> Self {
        let particles = (0..n).map(|_| random_particle(rng, &palette)).collect();
        Self { particles, palette }
    }

    pub fn step(&mut self, dt: f32) {
        let len = self.particles.len();
        for i in 0..len {
            for j in 0..len {
                if i != j {
                    let a = self.particles[i];
                    let b = self.particles[j];

                    // The vector pointing from a to b
                    let diff = b.pos - a.pos;

                    // Distance is capped
                    let dist = diff.magnitude();

                    // Accelerate towards b
                    let normal = diff.normalize();
                    let behav = self.palette.get_bahaviour(a.color, b.color);
                    let accel = normal * behav.interact(dist) / dist;

                    let vel = self.particles[i].vel + accel * dt;
                    let vel = vel * 0.9999;
                    self.particles[i].vel = vel;
                    self.particles[i].pos += vel * dt;
                }
            }
        }
    }
}

impl Palette {
    fn random_color(&self, rng: &mut Pcg) -> Color {
        (rng.gen_u32() as usize % self.colors.len()) as u8
    }

    pub fn get_bahaviour(&self, a: Color, b: Color) -> Behaviour {
        let idx = a as usize * self.colors.len() + b as usize;
        self.behaviours[idx]
    }
}

fn random_particle(rng: &mut Pcg, palette: &Palette) -> Particle {
    Particle {
        pos: Point2::new(rng.gen_f32(), rng.gen_f32()),
        vel: Vector2::zeros(),
        color: palette.random_color(rng),
    }
}

fn draw_particles(sim: &SimState) -> Mesh {
    let mut vertices = vec![];
    let indices = (0..sim.particles.len() as u32).collect();
    for particle in &sim.particles {
        let color = sim.palette.colors[particle.color as usize];

        let vertex = Vertex {
            pos: [particle.pos.x, 0., particle.pos.y],
            uvw: color,
        };

        vertices.push(vertex);
    }

    Mesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behaviour() {
        let behav = Behaviour {
            default_repulse: 1.0,
            inter_threshold: 0.25,
            inter_strength: 3.0,
            inter_max_dist: 0.75,
        };

        assert_eq!(behav.interact(0.), -behav.default_repulse);
        assert_eq!(behav.interact(behav.inter_threshold), 0.0);
        assert_eq!(behav.interact(0.25 + 0.125), behav.inter_strength / 2.);
        assert_eq!(behav.interact(0.5), behav.inter_strength);
        assert_eq!(behav.interact(behav.inter_max_dist), 0.0);
        assert_eq!(behav.interact(0.85), 0.0);
    }
}

impl Default for Behaviour {
    fn default() -> Self {
        Self {
            default_repulse: 10.,
            inter_threshold: 0.1,
            inter_strength: 1.,
            inter_max_dist: 3.,
        }
    }
}

impl Behaviour {
    pub fn with_inter_strength(mut self, inter_strength: f32) -> Self {
        self.inter_strength = inter_strength;
        self
    }
}
