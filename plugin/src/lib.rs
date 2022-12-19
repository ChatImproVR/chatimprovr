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

struct State {
    arcball: ArcBall,
    arcball_control: ArcBallController,
}

make_app_state!(State);

impl UserState for State {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create camera
        let camera_ent = io.create_entity();
        io.add_component(camera_ent, &Transform::default());
        io.add_component(camera_ent, &CameraComponent);

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

        Self {
            arcball: ArcBall::default(),
            arcball_control: ArcBallController::default(),
        }
    }
}

impl State {
    fn camera_move(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Handle input events
        if let Some(InputEvents(events)) = io.inbox_first() {
            for event in events {
                self.arcball_control.handle_event(&event, &mut self.arcball);
            }
        }

        // Set camera position
        for key in query.iter() {
            query.write::<Transform>(key, &self.arcball.camera_transf());
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
            closest_zoom: 2.,
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
