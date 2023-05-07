use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    glam::{Quat, Vec3},
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::camera::Perspective,
    vr::{ControllerEvent, ElementState, VrUpdate},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct Teleporter {
    left_hand: EntityId,
    right_hand: EntityId,
    path: Path,
    path_entity: EntityId,
    /// True if the path should be updated each frame
    update_path: bool,
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
            .query(
                "Camera",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    // Filter to camera component, so that we write to the camera's position
                    .intersect::<CameraComponent>(Access::Read),
            )
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
            mesh: path_mesh(&path_test, 100, [1.; 3]),
            id: PATH_RDR_ID,
        });

        Self {
            path: Path::default(),
            left_hand,
            right_hand,
            path_entity: path,
            update_path: false,
        }
    }
}

impl Teleporter {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let mut camera_transf = Transform::identity();
        for entity in query.iter("Camera") {
            camera_transf = query.read::<Transform>(entity);
        }

        // Handle events for VR
        let Some(update) = io.inbox_first::<VrUpdate>() else { return };

        if self.update_path {
            if let Some(grip) = update.right_controller.grip {
                let abs_pos = camera_transf * grip;

                self.path = Path::new(15., -9.8, abs_pos);
                let mesh = path_mesh(&self.path, 100, [1.; 3]);
                io.send(&UploadMesh {
                    id: PATH_RDR_ID,
                    mesh,
                });
            }
        }

        if update
            .right_controller
            .events
            .contains(&ControllerEvent::Trigger(ElementState::Pressed))
        {
            // Show path
            io.add_component(
                self.path_entity,
                Render::new(PATH_RDR_ID).primitive(Primitive::Lines),
            );
            self.update_path = true;
        }

        if update
            .right_controller
            .events
            .contains(&ControllerEvent::Trigger(ElementState::Released))
        {
            // Hide path
            io.add_component(self.path_entity, Render::new(PATH_RDR_ID).limit(Some(0)));

            self.update_path = false;

            for camera_entity in query.iter("Camera") {
                // Place the camera (reference frame) so that the new position has the left eye over
                // the desired end location
                let end_pos = self.path.sample(self.path.end_time());
                let left_eye = update.headset.left.transf;
                let destination = end_pos - Vec3::new(left_eye.pos.x, 0., left_eye.pos.z);
                io.add_component(camera_entity, Transform::new().with_position(destination));
            }
        }

        if let Some(pos) = update.left_controller.grip {
            io.add_component(self.left_hand, camera_transf * pos);
        }

        if let Some(pos) = update.right_controller.grip {
            io.add_component(self.right_hand, camera_transf * pos);
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

#[derive(Default, Copy, Clone, Debug)]
struct Quadratic {
    a: f32,
    b: f32,
    c: f32,
}

#[derive(Default, Copy, Clone, Debug)]
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
        let vect = hand.orient * Vec3::Z * throw_power;
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

fn path_mesh(path: &Path, samples: usize, color: [f32; 3]) -> Mesh {
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
