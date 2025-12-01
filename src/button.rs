use crate::{
    context::Context,
    dragged_card::CardDragState,
    renderer::{DrawCall, Hsl, Rgba, RichText, draw_rect, point_in_rect},
};

pub struct Button {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub color: Rgba,
    pub on_click: fn(&mut Context),
    pub enabled_when: fn(&Context) -> bool,
}

pub fn get_button_at(buttons: &[Button], x: u16, y: u16) -> Option<&Button> {
    for button in buttons {
        let button_x2: u16 = button.x + button.text.len() as u16 + 1;
        let button_y2: u16 = button.y;

        if point_in_rect(x, y, button.x, button.y, button_x2, button_y2) {
            return Some(button);
        };
    }

    None
}

pub fn draw_button(draw_queue: &mut Vec<DrawCall>, ctx: &Context, button: &Button) {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;

    draw_rect(
        draw_queue,
        button.x as i16,
        button.y as i16,
        w,
        h,
        button_bg(ctx, button),
    );
    draw_queue.push(DrawCall {
        x: button.x + 1,
        y: button.y,
        rich_text: RichText::new(&button.text)
            .with_fg(Rgba::from_f32(0.0, 0.0, 0.0, 1.0))
            .with_bold(true),
    })
}

fn button_bg(ctx: &Context, button: &Button) -> Rgba {
    let w: u16 = (button.text.len() + 2) as u16;
    let h: u16 = 1;
    let button_x2: u16 = button.x + w - 1;
    let button_y2: u16 = button.y + h - 1;

    let mut hsl: Hsl = button.color.into();

    let is_hovered: bool = point_in_rect(
        ctx.mouse.x,
        ctx.mouse.y,
        button.x,
        button.y,
        button_x2,
        button_y2,
    );
    let is_pressed: bool = is_hovered && ctx.mouse.is_left_down;
    let is_disabled: bool = !(button.enabled_when)(ctx);
    let is_dragging_anything: bool = matches!(ctx.mouse.card_drag, CardDragState::Dragging { .. });

    if is_disabled {
        hsl.l *= 0.4;
        hsl.s *= 0.4;
    } else if !is_dragging_anything {
        if is_pressed {
            hsl.l *= 0.3;
            hsl.s *= 0.3;
        } else if is_hovered {
            hsl.l += 0.08;
        }
    }

    hsl.into()
}
