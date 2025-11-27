use crate::renderer::{DrawCall, RichText};

pub struct FPSCounter {
    ema: f64,
    alpha: f64,
}

impl FPSCounter {
    pub fn new(alpha: f64) -> Self {
        Self { ema: 0.0, alpha }
    }

    pub fn update(&mut self, dt: f64) {
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

    pub fn fps(&self) -> f64 {
        self.ema
    }
}

pub fn draw_fps_counter(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, fps_counter: &FPSCounter) {
    draw_queue.push(DrawCall {
        x: x,
        y: y,
        rich_text: RichText::new(format!("FPS: {:2.0}", fps_counter.fps())),
    });
}
