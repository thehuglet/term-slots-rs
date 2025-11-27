use crate::{
    context::MouseContext,
    renderer::{DrawCall, HSL, RGBA, RichText, draw_rect, point_in_rect},
};

pub struct Button {
    pub x: u16,
    pub y: u16,
    pub text: &'static str,
    // pub action: Action,
    pub color: RGBA,
    pub disabled: bool,
}

pub fn draw_button(draw_queue: &mut Vec<DrawCall>, button: &Button, mouse: &MouseContext) {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;

    draw_rect(
        draw_queue,
        button.x,
        button.y,
        w,
        h,
        button_bg(&button, &mouse),
    );
    draw_queue.push(DrawCall {
        x: button.x + 1,
        y: button.y,
        rich_text: RichText::new(button.text)
            .with_fg(RGBA::from_f32(0.0, 0.0, 0.0, 1.0))
            .with_bold(true),
    })
}

fn button_bg(button: &Button, mouse: &MouseContext) -> RGBA {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;
    let button_x2: u16 = button.x + w - 1;
    let button_y2: u16 = button.y + h - 1;

    let mut hsl = HSL::from_rgba(button.color);

    let is_hovered: bool =
        point_in_rect(mouse.x, mouse.y, button.x, button.y, button_x2, button_y2);
    let is_pressed: bool = is_hovered && mouse.is_down;

    if button.disabled || is_pressed {
        hsl.l *= 0.5;
    } else if is_hovered {
        hsl.l *= 1.35;
    }

    RGBA::from_hsl(hsl)
}
