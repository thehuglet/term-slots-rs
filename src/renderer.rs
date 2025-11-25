#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl RGBA {
    // Create from 0.0..1.0 floats
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: a.clamp(0.0, 1.0),
        }
    }

    // Create from u8 values directly
    pub fn from_u8(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r,
            g,
            b,
            a: a.clamp(0.0, 1.0),
        }
    }

    // Simple linear interpolation
    pub fn lerp(&self, other: &RGBA, t: f32) -> RGBA {
        RGBA::from_f32(
            self.r as f32 / 255.0 * (1.0 - t) + other.r as f32 / 255.0 * t,
            self.g as f32 / 255.0 * (1.0 - t) + other.g as f32 / 255.0 * t,
            self.b as f32 / 255.0 * (1.0 - t) + other.b as f32 / 255.0 * t,
            self.a * (1.0 - t) + other.a * t,
        )
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
                a: 1.0,
            },
            bold: false,
        }
    }

    pub fn set_fg(mut self, fg: RGBA) -> Self {
        self.fg = fg;
        self
    }

    pub fn set_bg(mut self, bg: RGBA) -> Self {
        self.bg = bg;
        self
    }

    pub fn set_bold(mut self, value: bool) -> Self {
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
    cells: Vec<Cell>, // row-major
}

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub old_buffer: ScreenBuffer,
    pub new_buffer: ScreenBuffer,
}

impl Screen {
    pub fn new(width: usize, height: usize, default_bg: RGBA) -> Self {
        Self {
            width: width,
            height: height,
            old_buffer: ScreenBuffer::new(width, height, default_bg),
            new_buffer: ScreenBuffer::new(width, height, default_bg),
        }
    }

    pub fn swap_buffers(&mut self) {
        std::mem::swap(&mut self.old_buffer, &mut self.new_buffer);
    }

    pub fn diff_buffers(&self) -> Vec<(usize, usize, &Cell)> {
        let old = &self.old_buffer;
        let new = &self.new_buffer;
        let mut diffs = Vec::new();
        let h = old.height.min(new.height);
        let w = old.width.min(new.width);

        for y in 0..h {
            for x in 0..w {
                let a = &old.cells[y * old.width + x];
                let b = &new.cells[y * new.width + x];
                if a.ch != b.ch || a.fg != b.fg || a.bg != b.bg || a.bold != b.bold {
                    diffs.push((x, y, b)); // push the new buffer
                }
            }
        }

        diffs
    }
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

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&mut self.cells[y * self.width + x])
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&self.cells[y * self.width + x])
        }
    }
}

pub struct DrawCall {
    pub x: usize,
    pub y: usize,
    pub text: RichText,
}

pub fn compose_buffer(buf: &mut ScreenBuffer, draw_calls: &[DrawCall]) {
    for dc in draw_calls {
        let mut px = dc.x;
        let y = dc.y;

        let seg = &dc.text; // single RichText

        for ch in seg.text.chars() {
            if px >= buf.width || y >= buf.height {
                break;
            }

            let cell = &mut buf.cells[y * buf.width + px];
            cell.ch = ch;
            cell.fg = seg.fg;
            cell.bg = seg.bg;
            cell.bold = seg.bold;

            px += 1;
        }
    }
}

pub fn fill_screen_background(buf: &mut ScreenBuffer, bg: RGBA) {
    // optional default foreground
    for cell in buf.cells.iter_mut() {
        cell.ch = ' ';
        cell.fg = RGBA::from_u8(0, 0, 0, 1.0);
        cell.bg = bg;
        cell.bold = false;
    }
}
