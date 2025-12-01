use crate::renderer::{DrawCall, RichText};

pub struct FPSCounter {
    ema: f32,
    alpha: f32,
}

impl FPSCounter {
    pub fn new(alpha: f32) -> Self {
        Self { ema: 0.0, alpha }
    }

    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let inst = 1.0 / dt;
        if self.ema <= 0.0 {
            self.ema = inst;
        } else {
            self.ema = self.ema * (1.0 - self.alpha) + inst * self.alpha;
        }
    }

    pub fn fps(&self) -> f32 {
        self.ema
    }
}

pub fn draw_fps_counter(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, fps_counter: &FPSCounter) {
    draw_queue.push(DrawCall {
        x,
        y,
        rich_text: RichText::new(format!("FPS: {:2.0}", fps_counter.fps())),
    });
}
