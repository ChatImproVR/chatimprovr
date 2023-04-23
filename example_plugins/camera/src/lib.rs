use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{InputEvent, MouseButton},
    glam::{Mat3, Quat, Vec3, Vec4},
    render::{CameraComponent, Mesh, MeshHandle, Render, UploadMesh, Vertex},
    utils::{camera::Perspective, input_helper::InputHelper},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};

struct Camera {
    arcball: ArcBall,
    arcball_control: ArcBallController,
    proj: Perspective,
    left_hand: EntityId,
    right_hand: EntityId,
    input: InputHelper,
    /// Keep track of whether we've ever received a VR update, since we'll always receive desktop events!
    is_vr: bool,
}

make_app_state!(Camera, DummyUserState);

const HAND_RDR_ID: MeshHandle = MeshHandle::new(pkg_namespace!("Hand"));

impl UserState for Camera {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Create camera
        io.create_entity()
            .add_component(Transform::identity())
            .add_component(CameraComponent {
                clear_color: [0.; 3],
                projection: Default::default(),
            })
            .build();

        io.send(&hand());

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule
            .add_system(Self::update)
            .stage(Stage::PreUpdate)
            .subscribe::<InputEvent>()
            .subscribe::<VrUpdate>()
            .query(
                Query::new("Camera")
                    .intersect::<Transform>(Access::Write)
                    .intersect::<CameraComponent>(Access::Write),
            )
            .build();

        let left_hand = io
            .create_entity()
            .add_component(Render::new(HAND_RDR_ID))
            .build();

        let right_hand = io
            .create_entity()
            .add_component(Render::new(HAND_RDR_ID))
            .build();

        Self {
            input: InputHelper::new(),
            arcball: ArcBall::default(),
            arcball_control: ArcBallController::default(),
            proj: Perspective::new(),
            left_hand,
            right_hand,
            is_vr: false,
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

            self.is_vr = true;
            // Handle FOV changes (But NOT position. Position is extremely time-sensitive, so it
            // is actually prepended to the view matrix)
            self.proj.handle_vr_update(&update);

            if let Some(pos) = update.left_controller.grip {
                io.add_component(self.left_hand, pos);
            }

            if let Some(pos) = update.right_controller.grip {
                io.add_component(self.right_hand, pos);
            }
        }

        // Handle input events for desktop mode
        if !self.is_vr {
            for input in io.inbox::<InputEvent>() {
                // Handle window resizing
                self.proj.handle_event(&input);
            }

            self.input.handle_input_events(io);

            // Handle pivot/pan
            self.arcball_control.update(&self.input, &mut self.arcball);

            // Set camera transform to arcball position
            for key in query.iter("Camera") {
                query.write::<Transform>(key, &self.arcball.camera_transf());
            }
        }

        let projection = self.proj.matrices();

        let clear_color = [0.; 3];

        for key in query.iter("Camera") {
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
    pub pivot: Vec3,
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

    pub fn orient(&self) -> Quat {
        face_towards(self.eye(), Vec3::Y)
    }

    pub fn eye(&self) -> Vec3 {
        Vec3::new(
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
}

impl ArcBallController {
    pub fn update(&mut self, helper: &InputHelper, arcball: &mut ArcBall) {
        let (dx, dy) = helper.mouse_diff();

        if helper.mouse_held(MouseButton::Left) && !helper.held_shift() {
            self.pivot(arcball, dx, dy);
        } else if helper.mouse_held(MouseButton::Right)
            || (helper.mouse_held(MouseButton::Left) && helper.held_shift())
        {
            self.pan(arcball, dx, dy)
        }

        if let Some((_, dy)) = helper.mousewheel_scroll_diff() {
            self.zoom(arcball, dy);
        }
    }

    fn pivot(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32) {
        arcball.yaw += delta_x * self.swivel_sensitivity;
        arcball.pitch += delta_y * self.swivel_sensitivity;

        arcball.pitch = arcball.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    }

    fn pan(&mut self, arcball: &mut ArcBall, delta_x: f32, delta_y: f32) {
        let delta = Vec4::new(
            (-delta_x as f32) * arcball.distance,
            (delta_y as f32) * arcball.distance,
            0.0,
            0.0,
        ) * self.pan_sensitivity;

        // TODO: This is dumb, just use the cross product 4head
        let inv = arcball.camera_transf().to_homogeneous();
        arcball.pivot += (inv * delta).truncate();
    }

    fn zoom(&mut self, arcball: &mut ArcBall, delta: f32) {
        arcball.distance += delta * self.zoom_sensitivity.powf(2.) * arcball.distance;
        arcball.distance = arcball.distance.max(self.closest_zoom);
    }
}

impl Default for ArcBallController {
    fn default() -> Self {
        Self {
            pan_sensitivity: 0.0015,
            swivel_sensitivity: 0.005,
            zoom_sensitivity: 0.3,
            closest_zoom: 0.01,
        }
    }
}

impl Default for ArcBall {
    fn default() -> Self {
        Self {
            pivot: Vec3::new(0., 0., 0.),
            pitch: 0.3,
            yaw: 1.92,
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

// TODO: Add a PR to glam?
fn face_towards(dir: Vec3, up: Vec3) -> Quat {
    let zaxis = dir.normalize();
    let xaxis = up.cross(zaxis).normalize();
    let yaxis = zaxis.cross(xaxis).normalize();

    let mat = Mat3::from_cols(xaxis, yaxis, zaxis);

    Quat::from_mat3(&mat)
}
