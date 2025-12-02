use crate::renderer::{
    DrawCall, Hsl, Rgba, RichText, ScreenBuffer, blend_source_over, packed_rgb_to_rgba,
    rgba_to_packed_rgb,
};

pub fn build_vignette_lut(
    width: usize,
    height: usize,
    radius_scale: f32,
    falloff: f32,
    strength: f32,
) -> Vec<f32> {
    let mut lut: Vec<f32> = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let nx: f32 = (2.0 * x as f32) / (width as f32 * 2.0) - 0.5;
            let ny: f32 = (y as f32) / (height as f32) - 0.5;
            let d: f32 = (nx * nx + ny * ny).sqrt();
            let alpha: f32 = (d * radius_scale).powf(falloff).clamp(0.0, 1.0) * strength;
            lut.push(alpha);
        }
    }
    lut
}

// Agree to disagree, Clippy
#[allow(clippy::needless_range_loop)]
pub fn build_gamma_lut(gamma: f32) -> [u8; 256] {
    let mut lut = [0u8; 256];
    for i in 0..256 {
        lut[i] = ((i as f32 / 255.0).powf(gamma) * 255.0).round() as u8;
    }
    lut
}

pub fn apply_vignette(buf: &mut ScreenBuffer, lut: &[f32]) {
    let width: usize = buf.width as usize;
    let height: usize = buf.height as usize;
    for y in 0..height {
        for x in 0..width {
            let index: usize = y * width + x;
            let alpha: f32 = lut[index];

            let vignette_color = Rgba::from_u8(0, 0, 0, alpha);

            let old_bg = packed_rgb_to_rgba(buf.cells[index].bg);
            buf.cells[index].bg = rgba_to_packed_rgb(&blend_source_over(&old_bg, &vignette_color));
            let old_fg = packed_rgb_to_rgba(buf.cells[index].fg);
            buf.cells[index].fg = rgba_to_packed_rgb(&blend_source_over(&old_fg, &vignette_color));
        }
    }
}

pub fn apply_gamma(buffer: &mut ScreenBuffer, lut: &[u8; 256]) {
    for cell in buffer.cells.iter_mut() {
        let mut bg = packed_rgb_to_rgba(cell.bg);
        bg.r = lut[bg.r as usize];
        bg.g = lut[bg.g as usize];
        bg.b = lut[bg.b as usize];
        cell.bg = rgba_to_packed_rgb(&bg);

        let mut fg = packed_rgb_to_rgba(cell.fg);
        fg.r = lut[fg.r as usize];
        fg.g = lut[fg.g as usize];
        fg.b = lut[fg.b as usize];
        cell.fg = rgba_to_packed_rgb(&fg);
    }
}

pub fn draw_bg_shader(
    draw_queue: &mut Vec<DrawCall>,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    game_time: f32,
) {
    let origin_x: u16 = x;
    let origin_y: u16 = y;

    for x in origin_x..origin_x + w {
        for y in origin_y..origin_y + h {
            let base_color = Rgba::from_f32(0.0, 0.2, 0.0, 1.0);

            let frequency: f32 = 1.5;
            let amplitude: f32 = 0.018;

            let cell_x: u16 = x / 2;
            let cell_y: u16 = y;
            let is_checker: bool = (cell_x + cell_y) % 2 == 0;

            let offset: f32 = (cell_x as f32 / 8.0) + (cell_y as f32 / 4.0);
            let phase: f32 =
                ((cell_x as f32 * 12.9898 + cell_y as f32 * 78.233).sin() * 43_758.547).fract()
                    * std::f32::consts::PI;
            let t: f32 = game_time * frequency + offset + phase;

            let checker_dim_level: f32 = amplitude * t.sin();
            let dim_factor: f32 = if is_checker {
                1.0 + checker_dim_level
            } else {
                1.0 - checker_dim_level
            };

            let mut hsl: Hsl = base_color.into();
            hsl.l = hsl.l * dim_factor - 0.02;
            hsl.s *= 0.8;

            let base_swirl: f32 =
                ((cell_x as f32 * 0.3 + cell_y as f32 * 0.5 + t * 0.2).sin()) * 3.0;

            let rand_phase: f32 =
                ((cell_x as f32 * 12.34 + cell_y as f32 * 56.78).sin() * 43758.0).fract() * 3.0;

            hsl.h = (hsl.h + base_swirl + rand_phase) % 360.0;

            let color: Rgba = hsl.into();

            draw_queue.push(DrawCall {
                x,
                y,
                rich_text: RichText::new(" ")
                    .with_fg(Rgba::from_u8(0, 0, 0, 1.0))
                    .with_bg(color),
            });
        }
    }
}
