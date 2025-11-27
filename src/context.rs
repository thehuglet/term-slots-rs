use crate::{fps_counter::FPSCounter, renderer::Screen, slots::Slots};

pub struct Context {
    pub screen: Screen,
    pub mouse: MouseContext,
    pub game_time: f64,
    pub slots: Slots,
    pub fps_counter: FPSCounter,
}

pub struct MouseContext {
    pub x: u16,
    pub y: u16,
    pub is_down: bool,
}
