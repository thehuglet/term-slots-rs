use crate::{fps_counter::FPSCounter, renderer::Screen};

pub struct Context {
    pub screen: Screen,
    pub mouse_pos: (u16, u16),
    pub game_time: f32,
    pub fps_counter: FPSCounter,
}
