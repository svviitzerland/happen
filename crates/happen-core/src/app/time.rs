use std::time::Instant;

pub struct Time {
    pub delta: f64,
    pub delta_f32: f32,
    pub elapsed: f64,
    pub frame_count: u64,
    pub fixed_delta: f64,
    startup: Instant,
    last_frame: Instant,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            delta: 0.0,
            delta_f32: 0.0,
            elapsed: 0.0,
            frame_count: 0,
            fixed_delta: 1.0 / 60.0,
            startup: now,
            last_frame: now,
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.delta = (now - self.last_frame).as_secs_f64();
        self.delta_f32 = self.delta as f32;
        self.elapsed = (now - self.startup).as_secs_f64();
        self.frame_count += 1;
        self.last_frame = now;
    }

    pub fn fps(&self) -> f64 {
        if self.delta > 0.0 {
            1.0 / self.delta
        } else {
            0.0
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

