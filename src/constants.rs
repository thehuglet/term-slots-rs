use crate::renderer::Rgba;

pub const TERM_SCREEN_WIDTH: u16 = 54;
pub const TERM_SCREEN_HEIGHT: u16 = 30;

pub const BIG_PLAYING_CARD_WIDTH: u16 = 3;
pub const BIG_PLAYING_CARD_HEIGHT: u16 = 3;

pub const SLOTS_ORIGIN_X: u16 = 7;
pub const SLOTS_ORIGIN_Y: u16 = 5;
pub const SLOTS_COLUMNS_X_SPACING: u16 = 4;
pub const SLOTS_MAX_COLUMN_COUNT: u16 = 6;
pub const SLOTS_NEIGHBOR_ROW_COUNT: i16 = 3;

pub const HAND_ORIGIN_X: u16 = 5;
pub const HAND_ORIGIN_Y: u16 = 20;
pub const HAND_CARD_X_SPACING: u16 = 4;
pub const HAND_SLOT_COUNT: u16 = 7;

pub const TABLE_ORIGIN_X: u16 = 9;
pub const TABLE_ORIGIN_Y: u16 = 14;
pub const TABLE_CARD_X_SPACING: u16 = 4;
pub const TABLE_SLOT_COUNT: u16 = 5;

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
