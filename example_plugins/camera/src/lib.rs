use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{InputEvent, MouseButton},
    glam::{Mat3, Quat, Vec3, Vec4},
    render::CameraComponent,
    ui::GuiConfigMessage,
    utils::{camera::Perspective, input_helper::InputHelper},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{make_app_state, prelude::*};

struct Camera {
    arcball: ArcBall,
    arcball_control: ArcBallController,
    proj: Perspective,
    input: InputHelper,
    /// Keep track of whether we've ever received a VR update, since we'll always receive desktop events!
    is_vr: bool,
    is_tab_fullscreen: bool,
}

make_app_state!(Camera, DummyUserState);

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

        // Schedule the system
        // In the future it would be super cool to do this like Bevy and be able to just infer the
        // query from the type arguments and such...
        schedule
            .add_system(Self::update)
            .stage(Stage::PreUpdate)
            .subscribe::<InputEvent>()
            .subscribe::<VrUpdate>()
            .query(
                "Camera",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<CameraComponent>(Access::Write),
            )
            .build();

        Self {
            is_tab_fullscreen: false,
            input: InputHelper::new(),
            arcball: ArcBall::default(),
            arcball_control: ArcBallController::default(),
            proj: Perspective::new(),
            is_vr: false,
        }
    }
}

impl Camera {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Handle events for VR
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            if !self.is_vr {
                self.is_vr = true;

                // Sometimes the first InputEvent comes in before the first VrUpdate, and so we
                // need to reset the position back to zero here.
                for ent in query.iter("Camera") {
                    query.write(ent, &Transform::identity());
                }
            }
            // Handle FOV changes (But NOT position. Position is extremely time-sensitive, so it
            // is actually prepended to the view matrix)
            self.proj.handle_vr_update(&update);
        }

        // Handle input events for desktop mode
        if !self.is_vr {
            for input in io.inbox::<InputEvent>().collect::<Vec<_>>() {
                // Handle window resizing
                self.proj.handle_event(&input);
                self.handle_fullscreen_event(io, &input);
            }

            self.input.handle_input_events(io);

            // Handle pivot/pan
            self.arcball_control.update(&self.input, &mut self.arcball);

            // Set camera transform to arcball position
            for entity in query.iter("Camera") {
                query.write::<Transform>(entity, &self.arcball.camera_transf());
            }
        }

        let projection = self.proj.matrices();

        let clear_color = [0.; 3];

        for entity in query.iter("Camera") {
            query.write::<CameraComponent>(
                entity,
                &CameraComponent {
                    clear_color,
                    projection,
                },
            );
        }
    }

    pub fn handle_fullscreen_event(&mut self, io: &mut EngineIo, input: &InputEvent) {
        if input
            == &InputEvent::Keyboard(cimvr_common::desktop::KeyboardEvent::Key {
                key: cimvr_common::desktop::KeyCode::F,
                state: cimvr_common::desktop::ElementState::Released,
            })
        {
            self.is_tab_fullscreen = !self.is_tab_fullscreen;
            io.send(&GuiConfigMessage::TabFullscreen(self.is_tab_fullscreen));
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

// TODO: Submit a PR to glam?
fn face_towards(dir: Vec3, up: Vec3) -> Quat {
    let zaxis = dir.normalize();
    let xaxis = up.cross(zaxis).normalize();
    let yaxis = zaxis.cross(xaxis).normalize();

    let mat = Mat3::from_cols(xaxis, yaxis, zaxis);

    Quat::from_mat3(&mat)
}
