use cimvr_common::glam::Vec3;
use cimvr_engine_interface::pcg::Pcg;

use crate::query_accel::QueryAccelerator;

pub struct SimState {
    particles: Vec<Particle>,
    config: SimConfig,
    max_interaction_radius: f32,
    last_accel: QueryAccelerator,
    last_points: Vec<Vec3>,
}

type Color = u8;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub color: Color,
}

#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Debug)]
pub struct SimConfig {
    pub colors: Vec<[f32; 3]>,
    pub behaviours: Vec<Behaviour>,
    pub damping: f32,
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
    pub fn new(rng: &mut Pcg, config: SimConfig, n: usize) -> Self {
        let particles = (0..n).map(|_| random_particle(rng, &config)).collect();
        let max_interaction_radius: f32 = config
            .behaviours
            .iter()
            .map(|b| b.inter_max_dist)
            .fold(0., |r, acc| acc.max(r));

        Self {
            particles,
            config,
            max_interaction_radius,
            last_points: vec![],
            last_accel: QueryAccelerator::new(&[], 1.),
        }
    }

    pub fn move_neighbors(&mut self, pt: Vec3, accel: Vec3) {
        for i in self
            .last_accel
            .query_neighbors_by_point(&self.last_points, pt)
        {
            self.particles[i].vel += accel;
        }
    }

    pub fn step(&mut self, dt: f32) {
        let points: Vec<Vec3> = self.particles.iter().map(|p| p.pos).collect();
        let accel = QueryAccelerator::new(&points, self.max_interaction_radius);

        let len = self.particles.len();
        for i in 0..len {
            let mut total_accel = Vec3::ZERO;
            for neighbor in accel.query_neighbors(&points, i) {
                let a = self.particles[i];
                let b = self.particles[neighbor];

                // The vector pointing from a to b
                let diff = b.pos - a.pos;

                // Distance is capped
                let dist = diff.length();

                // Accelerate towards b
                let normal = diff.normalize();
                let behav = self.config.get_bahaviour(a.color, b.color);
                let accel = normal * behav.interact(dist) / dist;
                total_accel += accel;
            }

            let vel = self.particles[i].vel + total_accel * dt;

            // Dampen velocity
            let vel = vel * (1. - dt * self.config.damping);

            self.particles[i].vel = vel;
            self.particles[i].pos += vel * dt;
        }

        self.last_accel = accel;
        self.last_points = points;
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn config(&self) -> &SimConfig {
        &self.config
    }
}

impl SimConfig {
    fn random_color(&self, rng: &mut Pcg) -> Color {
        (rng.gen_u32() as usize % self.colors.len()) as u8
    }

    pub fn get_bahaviour(&self, a: Color, b: Color) -> Behaviour {
        let idx = a as usize * self.colors.len() + b as usize;
        self.behaviours[idx]
    }
}

fn random_particle(rng: &mut Pcg, config: &SimConfig) -> Particle {
    let range = 2.0;
    Particle {
        pos: Vec3::new(rng.gen_f32(), rng.gen_f32(), rng.gen_f32()) * range
            - Vec3::splat(range / 2.),
        vel: Vec3::ZERO,
        color: config.random_color(rng),
    }
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
            inter_threshold: 0.02,
            inter_strength: 1.,
            inter_max_dist: 0.2,
        }
    }
}

impl Behaviour {
    pub fn with_inter_strength(mut self, inter_strength: f32) -> Self {
        self.inter_strength = inter_strength;
        self
    }
}
