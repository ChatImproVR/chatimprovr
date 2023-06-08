use cimvr_common::{
    desktop::{InputEvent, KeyCode, MouseButton, WindowControl},
    glam::Vec3,
    utils::input_helper::InputHelper,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Default)]
struct ClientState {
    input: InputHelper,
    mouse_is_captured: bool,
}

#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
pub struct MoveCommand {
    pub distance: Vec3,
}

/// Component identifing the cube
#[derive(Component, Serialize, Deserialize, Default, Clone, Copy)]
pub struct CubeFlag;

make_app_state!(ClientState, DummyUserState);

impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<InputEvent>()
            .build();

        Self::default()
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.input.handle_input_events(io);

        if self.mouse_is_captured {
            dbg!(self.input.mouse_diff());
            if self.input.key_released(KeyCode::Escape) {
                io.send(&WindowControl::MouseRelease);
                self.mouse_is_captured = false;
            }
        } else {
            if self.input.mouse_pressed(MouseButton::Left) {
                io.send(&WindowControl::MouseCapture);
                self.mouse_is_captured = true;
            }
        }
    }
}
