use cimvr_engine::Engine;
use glutin::event::WindowEvent;

pub struct UserInputHandler {}

impl UserInputHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn publish_events(&mut self, engine: &mut Engine) {}

    pub fn handle_winit_event(&mut self, event: &WindowEvent) {}
}
