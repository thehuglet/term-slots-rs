use crossterm::style::{Attributes, Color, ContentStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Rgba {
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r,
            g,
            b,
            a: a.clamp(0.0, 1.0),
        }
    }
}

impl From<Hsl> for Rgba {
    fn from(hsl: Hsl) -> Self {
        let Hsl { h, s, l, a } = hsl;

        let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let hue_sector = h / 60.0;
        let x = chroma * (1.0 - (hue_sector % 2.0 - 1.0).abs());
        let lightness_adjustment = l - chroma / 2.0;

        let (red_component, green_component, blue_component) = match hue_sector {
            h if (0.0..1.0).contains(&h) => (chroma, x, 0.0),
            h if (1.0..2.0).contains(&h) => (x, chroma, 0.0),
            h if (2.0..3.0).contains(&h) => (0.0, chroma, x),
            h if (3.0..4.0).contains(&h) => (0.0, x, chroma),
            h if (4.0..5.0).contains(&h) => (x, 0.0, chroma),
            h if (5.0..6.0).contains(&h) => (chroma, 0.0, x),
            _ => (0.0, 0.0, 0.0), // fallback for invalid hue
        };

        Rgba {
            r: ((red_component + lightness_adjustment) * 255.0).round() as u8,
            g: ((green_component + lightness_adjustment) * 255.0).round() as u8,
            b: ((blue_component + lightness_adjustment) * 255.0).round() as u8,
            a, // Preserve alpha
        }
    }
}

/// Only use this for intermediate math, convert back to RGBA after you're done.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    // Preserves alpha channel when converting from RGBA
    pub a: f32,
}

impl From<Rgba> for Hsl {
    fn from(rgba: Rgba) -> Self {
        let Rgba { r, g, b, a } = rgba;

        let red = r as f32 / 255.0;
        let green = g as f32 / 255.0;
        let blue = b as f32 / 255.0;

        let max_component = red.max(green.max(blue));
        let min_component = red.min(green.min(blue));
        let delta = max_component - min_component;

        // Calculate lightness
        let lightness = (max_component + min_component) / 2.0;

        // Calculate hue
        let hue = if delta == 0.0 {
            0.0
        } else if max_component == red {
            60.0 * (((green - blue) / delta) % 6.0)
        } else if max_component == green {
            60.0 * (((blue - red) / delta) + 2.0)
        } else {
            60.0 * (((red - green) / delta) + 4.0)
        };

        let normalized_hue = if hue < 0.0 { hue + 360.0 } else { hue };

        // Calculate saturation
        let saturation = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lightness - 1.0).abs())
        };

        Hsl {
            h: normalized_hue,
            s: saturation,
            l: lightness,
            a, // Preserve alpha
        }
    }
}

pub struct RichText {
    pub text: String,
    pub fg: Rgba,
    pub bg: Rgba,
    pub bold: bool,
}

impl RichText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 1.0,
            },
            bg: Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0.0,
            },
            bold: false,
        }
    }

    pub fn with_fg(mut self, fg: Rgba) -> Self {
        self.fg = fg;
        self
    }

    pub fn with_bg(mut self, bg: Rgba) -> Self {
        self.bg = bg;
        self
    }

    pub fn with_bold(mut self, value: bool) -> Self {
        self.bold = value;
        self
    }
}

#[derive(Clone)]
pub struct Cell {
    pub ch: char,
    pub fg: u32, // Packed RGB: 0x00RRGGBB
    pub bg: u32, // Packed RGB: 0x00RRGGBB
    pub bold: bool,
}

pub struct ScreenBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl ScreenBuffer {
    fn new(width: u16, height: u16, default_bg: (u8, u8, u8)) -> Self {
        let cell = Cell {
            ch: ' ',
            fg: pack_rgb(255, 255, 255),
            bg: pack_rgb(default_bg.0, default_bg.1, default_bg.2),
            // fg: RGBA::from_u8(255, 255, 255, 1.0),
            // bg: default_bg,
            bold: false,
        };
        Self {
            width,
            height,
            cells: vec![cell; width as usize * height as usize],
        }
    }
}

pub struct Screen {
    pub old_buffer: ScreenBuffer,
    pub new_buffer: ScreenBuffer,
}

impl Screen {
    pub fn new(width: u16, height: u16, default_bg: (u8, u8, u8)) -> Self {
        Self {
            old_buffer: ScreenBuffer::new(width, height, default_bg),
            new_buffer: ScreenBuffer::new(width, height, default_bg),
        }
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.old_buffer, &mut self.new_buffer);
    }
}

pub struct DrawCall {
    pub x: u16,
    pub y: u16,
    pub rich_text: RichText,
}

pub fn point_in_rect(
    x: u16,
    y: u16,
    rect_x1: u16,
    rect_y1: u16,
    rect_x2: u16,
    rect_y2: u16,
) -> bool {
    x >= rect_x1 && x <= rect_x2 && y >= rect_y1 && y <= rect_y2
}

pub fn diff_buffers<'a>(old: &'a ScreenBuffer, new: &'a ScreenBuffer) -> Vec<(u16, u16, &'a Cell)> {
    let mut diffs = Vec::new();
    let h: usize = old.height.min(new.height) as usize;
    let w: usize = old.width.min(new.width) as usize;

    for y in 0..h {
        for x in 0..w {
            let old_cell = &old.cells[y * old.width as usize + x];
            let new_cell = &new.cells[y * new.width as usize + x];
            if old_cell.ch != new_cell.ch
                || old_cell.fg != new_cell.fg
                || old_cell.bg != new_cell.bg
                || old_cell.bold != new_cell.bold
            {
                diffs.push((x as u16, y as u16, new_cell));
            }
        }
    }

    diffs
}

pub fn compose_buffer(buf: &mut ScreenBuffer, draw_calls: &[DrawCall]) {
    for dc in draw_calls {
        let mut x: u16 = dc.x;
        let y: u16 = dc.y;

        for new_char in dc.rich_text.text.chars() {
            if x >= buf.width || y >= buf.height {
                break;
            }

            let cell: &mut Cell = &mut buf.cells[(y * buf.width + x) as usize];
            let new_rich_text: &RichText = &dc.rich_text;

            let is_old_char_visible: bool = cell.ch != ' ' && cell.fg != 0;
            let new_char_should_override_old: bool = new_rich_text.fg.a == 1.0;
            let preserve_old_bg: bool = new_rich_text.bg.a == 0.0;
            let skip_fg_blending: bool = dc.rich_text.fg.a == 1.0 || dc.rich_text.fg.a == 0.0;
            let skip_bg_blending: bool = dc.rich_text.bg.a == 1.0 || dc.rich_text.bg.a == 0.0;

            if new_char_should_override_old {
                cell.ch = new_char;
                cell.bold = new_rich_text.bold;

                if skip_fg_blending {
                    cell.fg = rgba_to_packed_rgb(&new_rich_text.fg);
                } else {
                    let bottom_color: Rgba = if is_old_char_visible {
                        packed_rgb_to_rgba(cell.fg)
                    } else {
                        packed_rgb_to_rgba(cell.bg)
                    };
                    let blended_fg = blend_source_over(&bottom_color, &new_rich_text.fg);
                    cell.fg = rgba_to_packed_rgb(&blended_fg);
                }
            } else if !skip_bg_blending {
                // Special case for no new char but new blended bg => tint the old fgq
                let old_fg: Rgba = packed_rgb_to_rgba(cell.fg);
                cell.fg = rgba_to_packed_rgb(&blend_source_over(&old_fg, &new_rich_text.bg))
            }

            if !preserve_old_bg {
                if skip_bg_blending {
                    cell.bg = rgba_to_packed_rgb(&new_rich_text.bg);
                } else {
                    let old_bg: Rgba = packed_rgb_to_rgba(cell.bg);
                    let blended_bg: Rgba = blend_source_over(&old_bg, &new_rich_text.bg);
                    cell.bg = rgba_to_packed_rgb(&blended_bg);
                }
            }

            x += 1;
        }
    }
}

pub fn fill_screen_background(buf: &mut ScreenBuffer, bg: (u8, u8, u8)) {
    for cell in buf.cells.iter_mut() {
        cell.ch = ' ';
        cell.fg = 0x000000; // Black
        cell.bg = pack_rgb(bg.0, bg.1, bg.2);
        cell.bold = false;
    }
}

/// Takes `i16` instead of `u16` for `x` and `y` to allow for
/// correct clipping when the drawn element is partially off screen.
pub fn draw_rect(draw_queue: &mut Vec<DrawCall>, x: i16, y: i16, w: u16, h: u16, color: Rgba) {
    for row_index in 0..h as i16 {
        let line_x: i16 = x.max(0);
        let line_y: i16 = y + row_index;

        let mut text_row: String = " ".repeat(w as usize);

        // x clipping
        if x < 0 {
            let chars_to_trim = -x as usize;
            let char_count = w as usize;

            if chars_to_trim >= char_count {
                continue;
            }

            text_row = text_row.chars().skip(chars_to_trim).collect::<String>()
        }

        // y clipping
        if line_y < 0 {
            continue;
        }

        draw_queue.push(DrawCall {
            x: line_x as u16,
            y: line_y as u16,
            rich_text: RichText::new(text_row)
                // This ensures the old buf char is drawn
                .with_fg(Rgba::from_u8(0, 0, 0, 0.0))
                .with_bg(color),
        })
    }
}

fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn unpack_rgb(packed: u32) -> (u8, u8, u8) {
    (
        ((packed >> 16) & 0xFF) as u8,
        ((packed >> 8) & 0xFF) as u8,
        (packed & 0xFF) as u8,
    )
}

pub fn rgba_to_packed_rgb(rgba: &Rgba) -> u32 {
    pack_rgb(rgba.r, rgba.g, rgba.b)
}

pub fn packed_rgb_to_rgba(packed_rgb: u32) -> Rgba {
    const ALPHA: f32 = 1.0;
    let (r, g, b) = unpack_rgb(packed_rgb);
    Rgba::from_u8(r, g, b, ALPHA)
}

pub fn blend_source_over(bottom: &Rgba, top: &Rgba) -> Rgba {
    let top_alpha = top.a.clamp(0.0, 1.0);
    let bottom_alpha = bottom.a.clamp(0.0, 1.0);

    let out_a = top_alpha + bottom_alpha * (1.0 - top_alpha);

    if out_a <= 0.0 {
        return Rgba::from_u8(0, 0, 0, 0.0);
    }

    let out_r = ((top.r as f32 / 255.0) * top_alpha
        + (bottom.r as f32 / 255.0) * bottom_alpha * (1.0 - top_alpha))
        / out_a;
    let out_g = ((top.g as f32 / 255.0) * top_alpha
        + (bottom.g as f32 / 255.0) * bottom_alpha * (1.0 - top_alpha))
        / out_a;
    let out_b = ((top.b as f32 / 255.0) * top_alpha
        + (bottom.b as f32 / 255.0) * bottom_alpha * (1.0 - top_alpha))
        / out_a;

    Rgba::from_f32(out_r, out_g, out_b, out_a)
}

pub fn build_crossterm_content_style(cell: &Cell) -> ContentStyle {
    let (fg_r, fg_g, fg_b) = unpack_rgb(cell.fg);
    let (bg_r, bg_g, bg_b) = unpack_rgb(cell.bg);

    let fg_color = Color::Rgb {
        r: fg_r,
        g: fg_g,
        b: fg_b,
    };

    let bg_color = Color::Rgb {
        r: bg_r,
        g: bg_g,
        b: bg_b,
    };

    let mut attrs = Attributes::none();
    if cell.bold {
        attrs = attrs.with(crossterm::style::Attribute::Bold);
    }

    ContentStyle {
        foreground_color: Some(fg_color),
        background_color: Some(bg_color),
        underline_color: None,
        attributes: attrs,
    }
}

// pub fn lerp_rgba(a: &Rgba, b: &Rgba, t: f32) -> Rgba {
//     Rgba::from_f32(
//         a.r as f32 / 255.0 * (1.0 - t) + b.r as f32 / 255.0 * t,
//         a.g as f32 / 255.0 * (1.0 - t) + b.g as f32 / 255.0 * t,
//         a.b as f32 / 255.0 * (1.0 - t) + b.b as f32 / 255.0 * t,
//         a.a * (1.0 - t) + b.a * t,
//     )
// }
