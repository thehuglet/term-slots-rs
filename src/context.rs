use crate::{
    constants::{SLOTS_MAX_COLUMN_COUNT, TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH},
    dragged_card::CardDragState,
    fps_counter::FPSCounter,
    hand::Hand,
    playing_card::standard_52_deck,
    renderer::Screen,
    shader::{build_gamma_lut, build_vignette_lut},
    slots::{Column, Slots},
    table::Table,
};

pub struct Context {
    pub settings: Settings,
    pub luts: LookUpTables,
    pub screen: Screen,
    pub mouse: MouseContext,
    pub game_time: f32,
    pub slots: Slots,
    pub table: Table,
    pub hand: Hand,
    pub fps_counter: FPSCounter,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            settings: Settings {
                vignette_enabled: true,
                bg_shader_enabled: true,
            },
            luts: LookUpTables {
                gamma: build_gamma_lut(0.75),
                vignette: build_vignette_lut(
                    TERM_SCREEN_WIDTH as usize,
                    TERM_SCREEN_HEIGHT as usize,
                    1.3,
                    2.0,
                    1.0,
                ),
            },
            screen: Screen::new(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT, (0, 0, 0)),
            mouse: MouseContext {
                x: 0,
                y: 0,
                is_left_down: false,
                card_drag: CardDragState::NotDragging,
            },
            game_time: 0.0,
            slots: Slots {
                columns: vec![
                    Column {
                        cursor: 0.0,
                        cards: standard_52_deck(),
                        spin_duration: 0.0,
                        spin_time_remaining: 0.0,
                        spin_speed: 0.0,
                    };
                    SLOTS_MAX_COLUMN_COUNT as usize
                ],
            },
            table: Table {
                cards_on_table: vec![],
            },
            hand: Hand {
                hand_size: 7,
                cards_in_hand: vec![],
            },
            fps_counter: FPSCounter::new(0.08),
        }
    }
}

pub struct MouseContext {
    pub x: u16,
    pub y: u16,
    pub is_left_down: bool,
    pub card_drag: CardDragState,
}

pub struct Settings {
    pub vignette_enabled: bool,
    pub bg_shader_enabled: bool,
}

pub struct LookUpTables {
    pub gamma: [u8; 256],
    pub vignette: Vec<f32>,
}
