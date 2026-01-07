use std::collections::HashMap;

use crate::{
    TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    card::standard_52_deck,
    card_ops::CardDragState,
    card_slot::{CardSlot, build_card_slots},
    fps_counter::FPSCounter,
    hand::{HAND_CARD_X_SPACING, HAND_ORIGIN_X, HAND_ORIGIN_Y, HAND_SLOT_COUNT},
    poker_hand::PokerHand,
    renderer::Screen,
    shader::{build_gamma_lut, build_vignette_lut},
    slot_machine::{SlotMachine, SlotMachineColumn},
    table::{TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_SLOT_COUNT},
};

pub struct Context {
    pub score: i32,
    pub coins: i32,
    pub luck: i32,
    pub game_time: f32,

    pub poker_hand: Option<PokerHand>,
    pub table_card_slots: Vec<CardSlot>,
    pub hand_card_slots: Vec<CardSlot>,
    pub slot_machine: SlotMachine,
    pub luts: LookUpTables,
    pub settings: Settings,
    pub mouse: MouseContext,
    pub screen: Screen,
    pub resize_update_accumulator: f32,
    pub impulse_timestamps: HashMap<ImpulseId, f32>,
    pub fps_counter: FPSCounter,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            score: 0,
            coins: 600,
            luck: 0,
            game_time: 0.0,
            impulse_timestamps: HashMap::new(),
            poker_hand: None,
            table_card_slots: build_card_slots(
                TABLE_ORIGIN_X,
                TABLE_ORIGIN_Y,
                TABLE_CARD_X_SPACING,
                TABLE_SLOT_COUNT.into(),
            ),
            hand_card_slots: build_card_slots(
                HAND_ORIGIN_X,
                HAND_ORIGIN_Y,
                HAND_CARD_X_SPACING,
                HAND_SLOT_COUNT.into(),
            ),
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
                    0.9,
                ),
            },
            screen: Screen::new(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT, (0, 0, 0)),
            mouse: MouseContext {
                x: 0,
                y: 0,
                is_left_down: false,
                card_drag: CardDragState::NotDragging,
            },
            slot_machine: SlotMachine {
                state: crate::slot_machine::SlotMachineState::Idle,
                spin_count: 0,
                columns: vec![
                    SlotMachineColumn {
                        cursor: 0.0,
                        cards: standard_52_deck(),
                        spin_duration: 0.0,
                        spin_time_remaining: 0.0,
                        spin_speed: 0.0,
                    };
                    6
                ],
            },
            resize_update_accumulator: 0.0,
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

#[derive(Hash, Eq, PartialEq)]
pub enum ImpulseId {
    NoSpaceInHandHint,
}
