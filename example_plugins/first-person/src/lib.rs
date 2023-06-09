use std::f32::consts::FRAC_PI_2;

use cimvr_common::{
    desktop::{InputEvent, KeyCode, MouseButton, WindowControl},
    glam::{EulerRot, Quat, Vec3},
    render::CameraComponent,
    utils::{camera::Perspective, input_helper::InputHelper},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, FrameTime};
use serde::{Deserialize, Serialize};

struct ClientState {
    input: InputHelper,
    mouse_is_captured: bool,
    proj: Perspective,
    cam_horiz_angle: f32,
    cam_vert_angle: f32,
    sensitivity_horiz: f32,
    sensitivity_vert: f32,
    last_resolution: Option<(u32, u32)>,
    default_speed: f32,
    sprint_speed: f32,
}

make_app_state!(ClientState, DummyUserState);

const CAM_HEIGHT: f32 = 1.8;

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<InputEvent>()
            .subscribe::<FrameTime>()
            .subscribe::<VrUpdate>()
            .query(
                "Camera",
                Query::new()
                    .intersect::<CameraComponent>(Access::Write)
                    .intersect::<Transform>(Access::Write),
            )
            .build();

        io.create_entity()
            .add_component(CameraComponent::default())
            .add_component(Transform::new().with_position(Vec3::new(0., CAM_HEIGHT, 5.)))
            .build();

        Self {
            input: InputHelper::new(),
            mouse_is_captured: false,
            proj: Perspective::new(),
            cam_vert_angle: 0.,
            cam_horiz_angle: 0.,
            sensitivity_horiz: 8e-4,
            sensitivity_vert: 8e-4,
            last_resolution: None,
            default_speed: 0.2,
            sprint_speed: 1.,
        }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Do nothing if in VR mode
        if io.inbox::<VrUpdate>().next().is_some() {
            return;
        }

        // Get entity
        let camera = query.iter("Camera").next().unwrap();

        // Handle projection matrices
        self.input.handle_input_events(io);
        for event in io.inbox::<InputEvent>() {
            self.proj.handle_event(&event);
        }
        query.modify::<CameraComponent>(camera, |cam| cam.projection = self.proj.matrices());

        // Handle mouse capturing
        if self.mouse_is_captured {
            if self.input.key_released(KeyCode::Escape) {
                io.send(&WindowControl::MouseRelease);
                self.mouse_is_captured = false;
            }
        } else {
            if self.input.mouse_pressed(MouseButton::Left) {
                io.send(&WindowControl::MouseCapture);
                self.mouse_is_captured = true;
                // Wait a frame for things to settle down
                return;
            }
        }

        let res @ (width, height) = self.input.get_resolution();

        if self.mouse_is_captured {
            // Handle camera swivel
            // Make sure resolution hasn't channged
            if Some(res) == self.last_resolution {
                if let Some((x, y)) = self.input.mouse_pos() {
                    // This is kinda jank lol (depends on desktop_input.rs in client...)
                    let dx = x - (width / 2) as f32;
                    let dy = y - (height / 2) as f32;

                    // Jank sanity check
                    let sanity = 425.;
                    if dx.abs() < sanity && dy.abs() < sanity {
                        self.cam_horiz_angle += dx * self.sensitivity_horiz;
                        self.cam_vert_angle += dy * self.sensitivity_vert;
                    }
                }

                //dbg!(self.cam_horiz_angle, self.cam_vert_angle);
                let quat = self.cam_orient();

                query.modify::<Transform>(camera, |tf| tf.orient = quat);
            }

            // Handle position update
            let mut movement_vector = Vec3::ZERO;

            if self.input.key_held(KeyCode::A) {
                movement_vector += Vec3::new(-1., 0., 0.);
            }
            if self.input.key_held(KeyCode::D) {
                movement_vector += Vec3::new(1., 0., 0.);
            }
            if self.input.key_held(KeyCode::W) {
                movement_vector += Vec3::new(0., 0., -1.);
            }
            if self.input.key_held(KeyCode::S) {
                movement_vector += Vec3::new(0., 0., 1.);
            }

            // Project movement vector onto camera orientation
            // You walk whichever direction you're looking
            let mut movement_vector = self.cam_orient() * movement_vector;
            movement_vector.y = 0.;

            let speed = if self.input.held_shift() {
                self.sprint_speed
            } else {
                self.default_speed
            };

            if self.input.key_held(KeyCode::Q) {
                movement_vector += Vec3::new(0., 1., 0.);
            }
            if self.input.key_held(KeyCode::E) {
                movement_vector += Vec3::new(0., -1., 0.);
            }

            let movement_vector = movement_vector.normalize_or_zero() * speed;
            query.modify::<Transform>(camera, |tf| tf.pos += movement_vector);
        }
        self.last_resolution = Some(res);
    }

    fn cam_orient(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.cam_horiz_angle, self.cam_vert_angle, 0.)
    }
}
