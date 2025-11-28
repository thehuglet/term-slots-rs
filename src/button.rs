use crate::{
    context::{Context, MouseContext},
    renderer::{DrawCall, HSL, RGBA, RichText, draw_rect, point_in_rect},
};

pub struct Button {
    pub x: u16,
    pub y: u16,
    pub text: &'static str,
    pub color: RGBA,
    pub on_click: fn(&mut Context),
    pub enabled_when: fn(&Context) -> bool,
}

pub fn draw_button(draw_queue: &mut Vec<DrawCall>, ctx: &Context, button: &Button) {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;

    draw_rect(
        draw_queue,
        button.x,
        button.y,
        w,
        h,
        button_bg(ctx, &button),
    );
    draw_queue.push(DrawCall {
        x: button.x + 1,
        y: button.y,
        rich_text: RichText::new(button.text)
            .with_fg(RGBA::from_f32(0.0, 0.0, 0.0, 1.0))
            .with_bold(true),
    })
}

fn button_bg(ctx: &Context, button: &Button) -> RGBA {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;
    let button_x2: u16 = button.x + w - 1;
    let button_y2: u16 = button.y + h - 1;

    let mut hsl: HSL = button.color.into();

    let is_hovered: bool = point_in_rect(
        ctx.mouse.x,
        ctx.mouse.y,
        button.x,
        button.y,
        button_x2,
        button_y2,
    );
    let is_pressed: bool = is_hovered && ctx.mouse.is_down;
    let is_disabled: bool = !(button.enabled_when)(ctx);

    if is_disabled {
        hsl.l *= 0.5;
        hsl.s *= 0.4;
    } else if is_pressed {
        hsl.l *= 0.65;
    } else if is_hovered {
        hsl.l *= 1.35;
    }

    hsl.into()
}
