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
    if fps_limiter.target_frametime == Duration::ZERO {
        let now: Instant = Instant::now();
        let dt: f32 = now
            .duration_since(fps_limiter.next_frame_timestamp - fps_limiter.target_frametime)
            .as_secs_f32();
        fps_limiter.next_frame_timestamp = now;
        return dt;
    }

    // Sleep until close to target
    while Instant::now() + fps_limiter.spin_reserve_sec < fps_limiter.next_frame_timestamp {
        let remaining: Duration =
            fps_limiter.next_frame_timestamp - Instant::now() - fps_limiter.spin_reserve_sec;
        sleep(fps_limiter.poll_interval_sec.min(remaining));
    }

    // Busy wait at the end for precision
    while Instant::now() < fps_limiter.next_frame_timestamp {
        std::hint::spin_loop();
    }

    let dt: f32 = Instant::now()
        .duration_since(fps_limiter.next_frame_timestamp - fps_limiter.target_frametime)
        .as_secs_f32();

    let frame_is_late: bool = Instant::now() > fps_limiter.next_frame_timestamp;
    fps_limiter.next_frame_timestamp = if frame_is_late {
        Instant::now() + fps_limiter.target_frametime
    } else {
        fps_limiter.next_frame_timestamp + fps_limiter.target_frametime
    };

    dt
}
