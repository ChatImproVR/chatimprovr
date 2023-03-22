use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    glam::{Quat, Vec3},
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::camera::Perspective,
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct Teleporter {
    left_hand: EntityId,
    right_hand: EntityId,
    path: EntityId,
}

make_app_state!(Teleporter, DummyUserState);

const HAND_RDR_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Hand"));
const PATH_RDR_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Path"));

impl UserState for Teleporter {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        io.send(&hand());

        let path = io
            .create_entity()
            .add_component(Transform::identity())
            .add_component(Render::new(PATH_RDR_ID).primitive(Primitive::Lines))
            .build();

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule
            .add_system(Self::update)
            .stage(Stage::PreUpdate)
            .subscribe::<VrUpdate>()
            .query::<Transform>(Access::Write)
            // Filter to camera component, so that we write to the camera's position
            .query::<CameraComponent>(Access::Read)
            .build();

        let left_hand = io
            .create_entity()
            .add_component(Render::new(HAND_RDR_ID).primitive(Primitive::Lines))
            .build();

        let right_hand = io
            .create_entity()
            .add_component(Render::new(HAND_RDR_ID).primitive(Primitive::Lines))
            .build();

        let path_test = Path::new(
            1.,
            -1.8,
            Transform {
                pos: Vec3::new(0., 2., 0.),
                orient: Quat::from_euler(cimvr_common::glam::EulerRot::XZY, 0., 0., 0.),
            },
        );
        io.send(&UploadMesh {
            mesh: render_path(&path_test, 100, [1.; 3]),
            id: PATH_RDR_ID,
        });

        Self {
            left_hand,
            right_hand,
            path,
        }
    }
}

impl Teleporter {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Handle events for VR
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            if !update.left_controller.events.is_empty() {
                dbg!(&update.left_controller.events);
            }

            if !update.right_controller.events.is_empty() {
                dbg!(&update.right_controller.events);
            }

            /*
            if let Some(camera_ent) = query.iter().next() {
                io.add_component(
                    camera_ent,
                    Transform::new()
                        .with_position()
                )
            }
            */

            if let Some(pos) = update.left_controller.grip {
                io.add_component(self.left_hand, pos);
            }

            if let Some(pos) = update.left_controller.grip {
                io.add_component(self.right_hand, pos);
            }
        }
    }
}

fn hand() -> UploadMesh {
    let s = 0.15;

    let vertices = vec![
        Vertex::new([0., 0., 0.], [1., 0., 0.]),
        Vertex::new([s, 0., 0.], [1., 0., 0.]),
        Vertex::new([0., 0., 0.], [0., 1., 0.]),
        Vertex::new([0., s, 0.], [0., 1., 0.]),
        Vertex::new([0., 0., 0.], [0., 0., 1.]),
        Vertex::new([0., 0., s], [0., 0., 1.]),
    ];

    let indices = vec![0, 1, 2, 3, 4, 5];

    UploadMesh {
        mesh: Mesh { vertices, indices },
        id: HAND_RDR_ID,
    }
}

struct Quadratic {
    a: f32,
    b: f32,
    c: f32,
}

struct Path {
    /// Quadratic function for the height
    quad: Quadratic,
    /// Direction along the controller's grip, multiplied by throw_power
    vect: Vec3,
    /// Position of the controller
    origin: Vec3,
    /// End "time"
    end_t: f32,
}

impl Path {
    pub fn new(throw_power: f32, g: f32, hand: Transform) -> Self {
        let vect = hand.orient * Vec3::NEG_Z * throw_power;
        let origin = hand.pos;

        let quad = Quadratic {
            a: g,
            b: vect.y,
            c: origin.y,
        };

        let end_t = quad.solve().unwrap_or(0.);

        Self {
            quad,
            vect,
            origin,
            end_t,
        }
    }

    /// End "time"
    pub fn end_time(&self) -> f32 {
        self.end_t
    }

    /// Sample a position along the path
    pub fn sample(&self, t: f32) -> Vec3 {
        Vec3::new(
            self.vect.x * t + self.origin.x,
            self.quad.sample(t),
            self.vect.z * t + self.origin.z,
        )
    }
}

impl Quadratic {
    pub fn solve(&self) -> Option<f32> {
        let b2 = self.b.powi(2);
        let ac4 = 4. * self.a * self.c;

        if b2 > ac4 {
            Some((-self.b + (b2 - ac4).sqrt()) / (2. * self.a))
        } else {
            None
        }
    }

    pub fn sample(&self, t: f32) -> f32 {
        self.a * t.powi(2) + self.b * t + self.c
    }
}

fn render_path(path: &Path, samples: usize, color: [f32; 3]) -> Mesh {
    let mut mesh = Mesh::new();

    for i in 0..=samples {
        let t = path.end_time() * i as f32 / samples as f32;
        let pos = path.sample(t);
        mesh.push_vertex(Vertex::new(pos.into(), color));
    }

    for i in 0..samples as u32 {
        mesh.indices.push(i);
        mesh.indices.push(i + 1);
    }

    mesh
}
