use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct FPSLimiter {
    target: Duration,
    next_frame: Instant,
    poll_interval: Duration,
    spin_reserve: Duration,
}

impl FPSLimiter {
    pub fn new(fps: f64, poll_interval_sec: f64, spin_reserve_sec: f64) -> Self {
        let fps_is_uncapped: bool = fps <= 0.0;

        let target = if fps_is_uncapped {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1.0 / fps)
        };

        let now = Instant::now();

        Self {
            target,
            next_frame: now + target,
            poll_interval: Duration::from_secs_f64(poll_interval_sec),
            spin_reserve: Duration::from_secs_f64(spin_reserve_sec),
        }
    }

    pub fn wait(&mut self) -> f64 {
        if self.target == Duration::ZERO {
            // Uncapped -> Return delta since last call
            let now = Instant::now();
            let dt = now
                .duration_since(self.next_frame - self.target)
                .as_secs_f64();
            self.next_frame = now;
            return dt;
        }

        let target_time = self.next_frame;
        let mut now = Instant::now();

        // Sleep until close to target
        while now + self.spin_reserve < target_time {
            let remaining = target_time - now - self.spin_reserve;
            sleep(self.poll_interval.min(remaining));
            now = Instant::now();
        }

        // Spin for final precision
        while Instant::now() < target_time {}

        let end = Instant::now();

        let dt = end
            .duration_since(self.next_frame - self.target)
            .as_secs_f64();

        // Schedule next frame
        self.next_frame = target_time + self.target;

        if end > self.next_frame {
            self.next_frame = end + self.target;
        }

        dt
    }
}
