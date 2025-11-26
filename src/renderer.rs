use crossterm::style::{Attributes, Color, ContentStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl RGBA {
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

pub struct RichText {
    text: String,
    fg: RGBA,
    bg: RGBA,
    bold: bool,
}

impl RichText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: RGBA {
                r: 255,
                g: 255,
                b: 255,
                a: 1.0,
            },
            bg: RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0.0,
            },
            bold: false,
        }
    }

    pub fn with_fg(mut self, fg: RGBA) -> Self {
        self.fg = fg;
        self
    }

    pub fn with_bg(mut self, bg: RGBA) -> Self {
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
    pub fg: RGBA,
    pub bg: RGBA,
    pub bold: bool,
}

pub struct ScreenBuffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize, default_bg: RGBA) -> Self {
        let cell = Cell {
            ch: ' ',
            fg: RGBA::from_u8(255, 255, 255, 1.0),
            bg: default_bg,
            bold: false,
        };
        Self {
            width,
            height,
            cells: vec![cell; width * height],
        }
    }
}

pub struct Screen {
    pub old_buffer: ScreenBuffer,
    pub new_buffer: ScreenBuffer,
}

impl Screen {
    pub fn new(width: usize, height: usize, default_bg: RGBA) -> Self {
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
    pub x: usize,
    pub y: usize,
    pub text: RichText,
}

pub fn diff_buffers<'a>(
    old: &'a ScreenBuffer,
    new: &'a ScreenBuffer,
) -> Vec<(usize, usize, &'a Cell)> {
    let mut diffs = Vec::new();
    let h = old.height.min(new.height);
    let w = old.width.min(new.width);

    for y in 0..h {
        for x in 0..w {
            let old_cell = &old.cells[y * old.width + x];
            let new_cell = &new.cells[y * new.width + x];
            if old_cell.ch != new_cell.ch
                || old_cell.fg != new_cell.fg
                || old_cell.bg != new_cell.bg
                || old_cell.bold != new_cell.bold
            {
                diffs.push((x, y, new_cell));
            }
        }
    }

    diffs
}

pub fn compose_buffer(buf: &mut ScreenBuffer, draw_calls: &[DrawCall]) {
    for dc in draw_calls {
        let mut px: usize = dc.x;
        let y: usize = dc.y;

        let new_seg: &RichText = &dc.text;

        for new_seg_ch in new_seg.text.chars() {
            if px >= buf.width || y >= buf.height {
                break;
            }

            let cell: &mut Cell = &mut buf.cells[y * buf.width + px];

            let skip_fg_blending: bool = new_seg_ch == ' ' || new_seg.fg.a == 1.0;
            let skip_bg_blending: bool =
                cell.bg.a == 0.0 || new_seg.bg.a == 1.0 || new_seg.bg.a == 0.0;
            let preserve_old_bg: bool = new_seg.bg.a == 0.0;

            if skip_fg_blending {
                cell.fg = new_seg.fg
            } else {
                // Blending with fg when the char is missing results in a more natural transition for certain effects,
                // especially useful for pop-up text as going from alpha 0.0 -> old cell will look more natural
                let bottom_col: RGBA;

                if cell.ch != ' ' {
                    bottom_col = cell.fg;
                } else {
                    bottom_col = cell.bg
                }

                cell.fg = blend_source_over(&bottom_col, &new_seg.fg);
            }

            cell.ch = new_seg_ch;
            cell.bold = new_seg.bold;

            if skip_bg_blending {
                if !preserve_old_bg {
                    cell.bg = new_seg.bg
                }
            } else {
                cell.bg = blend_source_over(&cell.bg, &new_seg.bg);
            }

            px += 1;
        }
    }
}

pub fn fill_screen_background(buf: &mut ScreenBuffer, bg: RGBA) {
    for cell in buf.cells.iter_mut() {
        cell.ch = ' ';
        cell.fg = RGBA::from_u8(0, 0, 0, 1.0);
        cell.bg = bg;
        cell.bold = false;
    }
}

/// Helper for drawing rectangles
pub fn draw_rect(
    draw_queue: &mut Vec<DrawCall>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: RGBA,
) {
    for row in 0..h {
        draw_queue.push(DrawCall {
            x: x,
            y: y + row,
            text: RichText::new(" ".repeat(w)).with_bg(color),
        })
    }
}

pub fn blend_source_over(bottom: &RGBA, top: &RGBA) -> RGBA {
    let top_alpha = top.a.clamp(0.0, 1.0);
    let bottom_alpha = bottom.a.clamp(0.0, 1.0);

    let out_a = top_alpha + bottom_alpha * (1.0 - top_alpha);

    if out_a <= 0.0 {
        return RGBA::from_u8(0, 0, 0, 0.0);
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

    RGBA::from_f32(out_r, out_g, out_b, out_a)
}

pub fn build_crossterm_content_style(cell: &Cell) -> ContentStyle {
    let fg_color = Color::Rgb {
        r: cell.fg.r,
        g: cell.fg.g,
        b: cell.fg.b,
    };

    let bg_color = Color::Rgb {
        r: cell.bg.r,
        g: cell.bg.g,
        b: cell.bg.b,
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

pub fn lerp_rgba(a: &RGBA, b: &RGBA, t: f32) -> RGBA {
    RGBA::from_f32(
        a.r as f32 / 255.0 * (1.0 - t) + b.r as f32 / 255.0 * t,
        a.g as f32 / 255.0 * (1.0 - t) + b.g as f32 / 255.0 * t,
        a.b as f32 / 255.0 * (1.0 - t) + b.b as f32 / 255.0 * t,
        a.a * (1.0 - t) + b.a * t,
    )
}
