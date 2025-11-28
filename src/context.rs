use crate::{
    dragged_card::DraggedCardContext, fps_counter::FPSCounter, hand::Hand, renderer::Screen,
    slots::Slots, table::Table,
};

pub struct Context {
    pub screen: Screen,
    pub mouse: MouseContext,
    pub dragged_card_ctx: DraggedCardContext,
    pub game_time: f64,
    pub slots: Slots,
    pub table: Table,
    pub hand: Hand,
    pub fps_counter: FPSCounter,
}

pub struct MouseContext {
    pub x: u16,
    pub y: u16,
    pub is_down: bool,
}
