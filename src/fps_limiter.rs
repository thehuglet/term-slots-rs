use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct FPSLimiter {
    target_frametime: Duration,
    next_frame_timestamp: Instant,
    poll_interval_sec: Duration,
    spin_reserve_sec: Duration,
}

impl FPSLimiter {
    pub fn new(fps: f32, poll_interval_sec: f32, spin_reserve_sec: f32) -> Self {
        let fps_is_uncapped: bool = fps <= 0.0;

        let target: Duration = if fps_is_uncapped {
            Duration::ZERO
        } else {
            Duration::from_secs_f32(1.0 / fps)
        };

        let now = Instant::now();

        Self {
            target_frametime: target,
            next_frame_timestamp: now + target,
            poll_interval_sec: Duration::from_secs_f32(poll_interval_sec),
            spin_reserve_sec: Duration::from_secs_f32(spin_reserve_sec),
        }
    }
}

pub fn wait_for_next_frame(fps_limiter: &mut FPSLimiter) -> f32 {
    let is_uncapped: bool = fps_limiter.target_frametime == Duration::ZERO;
    if is_uncapped {
        let now: Instant = Instant::now();
        let dt: f32 = now
            .duration_since(fps_limiter.next_frame_timestamp - fps_limiter.target_frametime)
            .as_secs_f32();
        fps_limiter.next_frame_timestamp = now;
        return dt;
    }

    let target_time: Instant = fps_limiter.next_frame_timestamp;
    let mut now: Instant = Instant::now();

    // Sleep until close to target
    while now + fps_limiter.spin_reserve_sec < target_time {
        let remaining: Duration = target_time - now - fps_limiter.spin_reserve_sec;
        sleep(fps_limiter.poll_interval_sec.min(remaining));
        now = Instant::now();
    }

    // Busy wait at the end for precision
    while Instant::now() < target_time {
        std::hint::spin_loop();
    }

    let end: Instant = Instant::now();

    let dt: f32 = end
        .duration_since(fps_limiter.next_frame_timestamp - fps_limiter.target_frametime)
        .as_secs_f32();

    fps_limiter.next_frame_timestamp = target_time + fps_limiter.target_frametime;

    let frame_is_late: bool = end > fps_limiter.next_frame_timestamp;
    if frame_is_late {
        fps_limiter.next_frame_timestamp = end + fps_limiter.target_frametime;
    }

    dt
}
