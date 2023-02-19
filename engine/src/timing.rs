use std::time::Instant;

use cimvr_engine_interface::FrameTime;

pub struct Timing {
    init: Instant,
    last_frame: Instant,
}

impl Timing {
    pub fn init() -> Self {
        let init = Instant::now();
        Self {
            last_frame: init,
            init,
        }
    }

    /// Begin the frame, as far as this clock is concerned.
    pub fn frame(&mut self) -> FrameTime {
        let frame_start = Instant::now();
        let delta = frame_start - self.last_frame;
        let time = self.init - frame_start;
        FrameTime {
            delta: delta.as_secs_f32(),
            time: time.as_secs_f32(),
        }
    }
}
