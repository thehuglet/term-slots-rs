use crate::renderer::Rgba;

pub const SIDEBAR_BORDER_X: u16 = 37;

pub static CARD_SLOT_COLOR: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0.4,
};

pub static SUIT_COLOR_BLACK: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 1.0,
};

pub static SUIT_COLOR_RED: Rgba = Rgba {
    r: 200,
    g: 0,
    b: 0,
    a: 1.0,
};

pub static DEFAULT_CARD_BG_COLOR: Rgba = Rgba {
    r: 255,
    g: 255,
    b: 255,
    a: 1.0,
};
