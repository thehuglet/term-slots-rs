use crate::{
    input::Action,
    renderer::{DrawCall, RGBA, RichText, draw_rect, point_in_rect},
};

pub struct Button {
    pub x: usize,
    pub y: usize,
    pub text: &'static str,
    pub action: Action,
    pub color: RGBA,
}

pub fn draw_button(
    draw_queue: &mut Vec<DrawCall>,
    button: &Button,
    mouse_x: usize,
    mouse_y: usize,
) {
    let w: usize = button.text.len() + 2;
    let h: usize = 1;
    let button_x2 = button.x + w - 1;
    let button_y2 = button.y + h - 1;

    let is_hovering = point_in_rect(mouse_x, mouse_y, button.x, button.y, button_x2, button_y2);

    let bg_color = if is_hovering {
        RGBA::from_f32(0.9, 0.9, 0.9, 1.0) // highlighted
    } else {
        RGBA::from_f32(0.6, 0.6, 0.6, 1.0) // normal
    };

    draw_rect(draw_queue, button.x, button.y, w, h, bg_color);

    draw_queue.push(DrawCall {
        x: button.x + 1,
        y: button.y,
        text: RichText::new(button.text)
            .with_fg(RGBA::from_f32(0.0, 0.0, 0.0, 1.0))
            .with_bold(true),
    })
}
