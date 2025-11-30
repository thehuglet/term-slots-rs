use crate::renderer::RGBA;

pub const TERM_SCREEN_WIDTH: u16 = 50;
pub const TERM_SCREEN_HEIGHT: u16 = 30;

pub const TABLE_SLOT_COUNT: u16 = 5;
pub const MAX_SLOTS_COLUMNS: u16 = 6;

pub const SLOTS_ORIGIN_X: u16 = 5;
pub const SLOTS_ORIGIN_Y: u16 = 5;

pub const TABLE_ORIGIN_X: u16 = 6;
pub const TABLE_ORIGIN_Y: u16 = 14;

pub const HAND_ORIGIN_X: u16 = 5;
pub const HAND_ORIGIN_Y: u16 = 20;

pub const SLOTS_COLUMNS_X_SPACING: u16 = 4;
pub const TABLE_CARD_X_SPACING: u16 = 4;
pub const HAND_CARD_X_SPACING: u16 = 4;

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
