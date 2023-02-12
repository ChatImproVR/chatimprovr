use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{
        ElementState, InputEvent, InputEvents, KeyboardEvent, ModifiersState, MouseButton,
        MouseEvent, WindowEvent,
    },
    nalgebra::{Matrix4, Point3, UnitQuaternion, Vector3, Vector4},
    render::{CameraComponent, Mesh, MeshHandle, Render, UploadMesh, Vertex},
    vr::{VrFov, VrUpdate},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct ClientState {
    arcball: ArcBall,
    arcball_control: ArcBallController,
    screen_size: (u32, u32),
    left_hand: EntityId,
    right_hand: EntityId,
}

make_app_state!(ClientState, DummyUserState);

const HAND_RDR_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Hand"));

impl UserState for ClientState {
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
            screen_size: (1920, 1080),
            left_hand,
            right_hand,
        }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let clear_color = [0.; 3];

        // Handle input events for desktop mode
        if let Some(InputEvents(events)) = io.inbox_first() {
            for event in events {
                self.arcball_control.handle_event(&event, &mut self.arcball);
                if let InputEvent::Window(WindowEvent::Resized { width, height }) = event {
                    self.screen_size = (width, height);
                }
            }

            // Get projection matrix
            let proj = Matrix4::new_perspective(
                self.screen_size.0 as f32 / self.screen_size.1 as f32,
                45_f32.to_radians(),
                0.01,
                1000.,
            );

            // Set camera position
            for key in query.iter() {
                query.write::<Transform>(key, &self.arcball.camera_transf());
                query.write::<CameraComponent>(
                    key,
                    &CameraComponent {
                        clear_color,
                        projection: [proj, proj],
                    },
                );
            }
        }

        // Handle events for VR
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            // Set correct FOV for each eye
            let near = 0.01;
            let far = 1000.;
            let left_proj = vr_projection_from_fov(&update.fov_left, near, far);
            let right_proj = vr_projection_from_fov(&update.fov_right, near, far);

            for key in query.iter() {
                query.write::<CameraComponent>(
                    key,
                    &CameraComponent {
                        clear_color,
                        projection: [left_proj, right_proj],
                    },
                );
            }

            if let Some(pos) = update.grip_left {
                io.add_component(self.left_hand, &pos);
            }

            if let Some(pos) = update.grip_right {
                io.add_component(self.right_hand, &pos);
            }
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

/// Creates a projection matrix for the given fov
pub fn vr_projection_from_fov(fov: &VrFov, near: f32, far: f32) -> Matrix4<f32> {
    // See https://gitlab.freedesktop.org/monado/demos/openxr-simple-example/-/blob/master/main.c
    // XrMatrix4x4f_CreateProjectionFov()

    let tan_left = fov.angle_left.tan();
    let tan_right = fov.angle_right.tan();

    let tan_up = fov.angle_up.tan();
    let tan_down = fov.angle_down.tan();

    let tan_width = tan_right - tan_left;
    let tan_height = tan_up - tan_down;

    let a11 = 2.0 / tan_width;
    let a22 = 2.0 / tan_height;

    let a31 = (tan_right + tan_left) / tan_width;
    let a32 = (tan_up + tan_down) / tan_height;

    let a33 = -far / (far - near);

    let a43 = -(far * near) / (far - near);

    Matrix4::new(
        a11, 0.0, a31, 0.0, //
        0.0, a22, a32, 0.0, //
        0.0, 0.0, a33, a43, //
        0.0, 0.0, -1.0, 0.0, //
    )
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
