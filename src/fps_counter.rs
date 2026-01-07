use crate::renderer::{DrawCall, RichText};

pub struct FPSCounter {
    fps_ema: f32,
    smoothing_factor: f32,
}

impl FPSCounter {
    pub fn new(alpha: f32) -> Self {
        Self {
            fps_ema: 0.0,
            smoothing_factor: alpha,
        }
    }
}

pub fn update_fps_counter(fps_counter: &mut FPSCounter, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    let current_fps: f32 = 1.0 / dt;

    if fps_counter.fps_ema <= 0.0 {
        fps_counter.fps_ema = current_fps;
    } else {
        fps_counter.fps_ema = fps_counter.fps_ema * (1.0 - fps_counter.smoothing_factor)
            + current_fps * fps_counter.smoothing_factor;
    }
}

pub fn draw_fps_counter(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, fps_counter: &FPSCounter) {
    draw_queue.push(DrawCall {
        x,
        y,
        rich_text: RichText::new(format!("FPS: {:2.0}", fps_counter.fps_ema)),
    });
}
