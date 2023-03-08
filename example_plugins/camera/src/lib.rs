use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{
        ElementState, InputEvent, InputEvents, KeyboardEvent, ModifiersState, MouseButton,
        MouseEvent,
    },
    nalgebra::{Point3, UnitQuaternion, Vector3, Vector4},
    render::{CameraComponent, Mesh, MeshHandle, Render, UploadMesh, Vertex},
    utils::camera::Perspective,
    vr::{VrUpdate},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct Camera {
    arcball: ArcBall,
    arcball_control: ArcBallController,
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
        io.add_component(camera_ent, &Transform::identity());
        io.add_component(
            camera_ent,
            &CameraComponent {
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
                .subscribe::<InputEvents>()
                .subscribe::<VrUpdate>()
                .query::<Transform>(Access::Write)
                .query::<CameraComponent>(Access::Write),
        );

        let left_hand = io.create_entity();
        let right_hand = io.create_entity();

        io.add_component(left_hand, &Render::new(HAND_RDR_ID));
        io.add_component(right_hand, &Render::new(HAND_RDR_ID));

        Self {
            arcball: ArcBall::default(),
            arcball_control: ArcBallController::default(),
            proj: Perspective::new(),
            left_hand,
            right_hand,
        }
    }
}

impl Camera {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Handle input events for desktop mode
        if let Some(input) = io.inbox_first::<InputEvents>() {
            // Handle window resizing
            self.proj.handle_input_events(&input);

            //dbg!(&input);

            // Handle pivot/pan
            for event in input.0 {
                self.arcball_control.handle_event(&event, &mut self.arcball);
            }

            // Set camera transform to arcball position
            for key in query.iter() {
                query.write::<Transform>(key, &self.arcball.camera_transf());
            }
        }

        // Handle events for VR
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            // Handle FOV changes (But NOT position. Position is extremely time-sensitive, so it
            // is actually prepended to the view matrix)
            self.proj.handle_vr_update(&update);

            if let Some(pos) = update.grip_left {
                io.add_component(self.left_hand, &pos);
            }

            if let Some(pos) = update.grip_right {
                io.add_component(self.right_hand, &pos);
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

/// Arcball camera parameters
#[derive(Copy, Clone)]
pub struct ArcBall {
    pub pivot: Point3<f32>,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl ArcBall {
    pub fn camera_transf(&self) -> Transform {
        Transform {
            pos: self.pivot + self.eye(),
            orient: self.orient(),
        }
    }

    pub fn orient(&self) -> UnitQuaternion<f32> {
        UnitQuaternion::face_towards(&(self.eye()), &Vector3::y())
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos().abs(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos().abs(),
        ) * self.distance
    }
}

/// Arcball camera controller parameters
#[derive(Copy, Clone)]
pub struct ArcBallController {
    pub pan_sensitivity: f32,
    pub swivel_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub closest_zoom: f32,
    pub last_mouse: Option<(f32, f32)>,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub modifiers: ModifiersState,
}

impl ArcBallController {
    pub fn handle_event(&mut self, event: &InputEvent, arcball: &mut ArcBall) {
        match event {
            InputEvent::Mouse(mouse) => match mouse {
                MouseEvent::Moved(x, y) => {
                    if let Some((lx, ly)) = self.last_mouse {
                        let (dx, dy) = (x - lx, y - ly);

                        if self.mouse_left && !self.modifiers.shift {
                            self.pivot(arcball, dx, dy);
                        } else if self.mouse_right || (self.mouse_left && self.modifiers.shift) {
                            self.pan(arcball, dx, dy)
                        }
                    }

                    self.last_mouse = Some((*x, *y));
                }
                MouseEvent::Scrolled(_, dy) => {
                    self.zoom(arcball, *dy);
                }
                MouseEvent::Clicked(button, state, _) => {
                    let b = *state == ElementState::Pressed;
                    match button {
                        MouseButton::Left => self.mouse_left = b,
                        MouseButton::Right => self.mouse_right = b,
                        _ => (),
                    }
                }
                _ => (),
            },
            InputEvent::Keyboard(KeyboardEvent::Modifiers(modifiers)) => {
                self.modifiers = *modifiers;
            }
            _ => (),
        }
    }

    fn pivot(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32) {
        arcball.yaw += delta_x * self.swivel_sensitivity;
        arcball.pitch += delta_y * self.swivel_sensitivity;

        arcball.pitch = arcball.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    }

    fn pan(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32) {
        let delta = Vector4::new(
            (-delta_x as f32) * arcball.distance,
            (delta_y as f32) * arcball.distance,
            0.0,
            0.0,
        ) * self.pan_sensitivity;

        // TODO: This is dumb, just use the cross product 4head
        let inv = arcball.camera_transf().to_homogeneous();
        arcball.pivot += (inv * delta).xyz();
    }

    fn zoom(&mut self, arcball: &mut ArcBall, delta: f32) {
        arcball.distance += delta * self.zoom_sensitivity.powf(2.) * arcball.distance;
        arcball.distance = arcball.distance.max(self.closest_zoom);
    }
}

impl Default for ArcBallController {
    fn default() -> Self {
        Self {
            modifiers: ModifiersState::default(),
            pan_sensitivity: 0.0015,
            swivel_sensitivity: 0.005,
            zoom_sensitivity: 0.3,
            closest_zoom: 0.01,
            last_mouse: None,
            mouse_left: false,
            mouse_right: false,
        }
    }
}

impl Default for ArcBall {
    fn default() -> Self {
        Self {
            pivot: Point3::new(0., 0., 0.),
            pitch: 0.3,
            yaw: -1.92,
            distance: 10.,
        }
    }
}

fn hand() -> UploadMesh {
    let s = 0.15;

    let vertices = vec![
        Vertex::new([-s, -s, -s], [0.0, 1.0, 1.0]),
        Vertex::new([s, -s, -s], [1.0, 0.0, 1.0]),
        Vertex::new([s, s, -s], [1.0, 1.0, 0.0]),
        Vertex::new([-s, s, -s], [0.0, 1.0, 1.0]),
        Vertex::new([-s, -s, s], [1.0, 0.0, 1.0]),
        Vertex::new([s, -s, s], [1.0, 1.0, 0.0]),
        Vertex::new([s, s, s], [0.0, 1.0, 1.0]),
        Vertex::new([-s, s, s], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    UploadMesh {
        mesh: Mesh { vertices, indices },
        id: HAND_RDR_ID,
    }
}
