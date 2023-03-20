use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{
        ElementState, InputEvent, InputEvents, KeyboardEvent, ModifiersState, MouseButton,
        MouseEvent,
    },
    glam::{Mat3, Quat, Vec3, Vec4},
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::camera::Perspective,
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct Camera {
    proj: Perspective,
    left_hand: EntityId,
    right_hand: EntityId,
}

make_app_state!(Camera, DummyUserState);

const HAND_RDR_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Hand"));

impl UserState for Camera {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create camera
        let camera_ent = io.create_entity();
        io.add_component(camera_ent, Transform::identity());
        io.add_component(
            camera_ent,
            CameraComponent {
                clear_color: [0.; 3],
                projection: Default::default(),
            },
        );

        io.send(&hand());

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule.add_system(
            Self::update,
            SystemDescriptor::new(Stage::PreUpdate)
                .subscribe::<VrUpdate>()
                .query::<Transform>(Access::Write)
                .query::<CameraComponent>(Access::Write),
        );

        let left_hand = io.create_entity();
        let right_hand = io.create_entity();

        io.add_component(
            left_hand,
            Render::new(HAND_RDR_ID).primitive(Primitive::Lines),
        );
        io.add_component(
            right_hand,
            Render::new(HAND_RDR_ID).primitive(Primitive::Lines),
        );

        Self {
            proj: Perspective::new(),
            left_hand,
            right_hand,
        }
    }
}

impl Camera {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Handle events for VR
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            if !update.left_controller.events.is_empty() {
                dbg!(&update.left_controller.events);
            }

            if !update.right_controller.events.is_empty() {
                dbg!(&update.right_controller.events);
            }

            // Handle FOV changes (But NOT position. Position is extremely time-sensitive, so it
            // is actually prepended to the view matrix)
            self.proj.handle_vr_update(&update);

            if let Some(pos) = update.left_controller.grip {
                io.add_component(self.left_hand, pos);
            }

            if let Some(pos) = update.left_controller.grip {
                io.add_component(self.right_hand, pos);
            }
        }

        let projection = self.proj.matrices();

        let clear_color = [0.; 3];

        for key in query.iter() {
            query.write::<CameraComponent>(
                key,
                &CameraComponent {
                    clear_color,
                    projection,
                },
            );
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
