use crate::renderer::RGBA;

pub const HAND_ORIGIN_X: u16 = 5;
pub const HAND_ORIGIN_Y: u16 = 20;

pub static SUIT_COLOR_BLACK: RGBA = RGBA {
    r: 0,
    g: 0,
    b: 0,
    a: 1.0,
};

pub static SUIT_COLOR_RED: RGBA = RGBA {
    r: 200,
    g: 0,
    b: 0,
    a: 1.0,
};

pub static DEFAULT_CARD_BG_COLOR: RGBA = RGBA {
    r: 255,
    g: 255,
    b: 255,
    a: 1.0,
};
