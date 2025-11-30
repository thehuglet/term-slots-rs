use crate::{
    dragged_card::CardDragState, fps_counter::FPSCounter, hand::Hand, renderer::Screen,
    slots::Slots, table::Table,
};

pub struct Context {
    pub vignette_lut: Vec<f32>,
    pub gamma_lut: [u8; 256],
    pub gamma_correction: bool,
    pub vignette: bool,
    pub screen: Screen,
    pub mouse: MouseContext,
    // pub dragged_card_ctx: DraggedCardContext,
    pub game_time: f64,
    pub slots: Slots,
    pub table: Table,
    pub hand: Hand,
    pub fps_counter: FPSCounter,
}

pub struct MouseContext {
    pub x: u16,
    pub y: u16,
    pub is_left_down: bool,
    pub card_drag: CardDragState,
}
