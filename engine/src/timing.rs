use std::time::Instant;

use cimvr_engine_interface::FrameTime;

pub struct Timing {
    init: Instant,
    last_frame: Instant,
    time: FrameTime,
}

impl Timing {
    pub fn init() -> Self {
        let init = Instant::now();
        Self {
            last_frame: init,
            init,
            time: FrameTime {
                delta: 0.,
                time: 0.,
            },
        }
    }

    /// Begin the frame, as far as this clock is concerned.
    pub fn frame(&mut self) {
        let frame_start = Instant::now();
        let delta = frame_start - self.last_frame;
        let time = frame_start - self.init;
        self.last_frame = frame_start;

        self.time = FrameTime {
            delta: delta.as_secs_f32(),
            time: time.as_secs_f32(),
        };
    }

    /// Get the current frame timing
    pub fn time(&self) -> FrameTime {
        self.time
    }
}
