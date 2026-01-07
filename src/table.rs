use crate::{
    card::{BIG_CARD_HEIGHT, BIG_CARD_WIDTH, draw_calls_playing_card_big},
    card_ops::{CardDragAndDropLocation, CardDragState},
    constants::CARD_SLOT_COLOR,
    context::Context,
    renderer::{DrawCall, Hsl, Rgba, RichText, draw_rect},
};

pub const TABLE_ORIGIN_X: u16 = 9;
pub const TABLE_ORIGIN_Y: u16 = 14;
pub const TABLE_CARD_X_SPACING: u16 = 4;
pub const TABLE_SLOT_COUNT: u16 = 5;

pub fn draw_table(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, ctx: &Context) {
    let slots_with_cards = ctx
        .table_card_slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| match &slot.card {
            Some(card) => Some((index, slot, card)),
            None => None,
        });

    for (index, slot, card) in slots_with_cards {
        let is_being_dragged = matches!(ctx.mouse.card_drag,
            CardDragState::Dragging {
                source: CardDragAndDropLocation::Table { index: src_index },
                ..
            } if src_index == index
        );

        if is_being_dragged {
            // Skip drawing the card at the slot
            // while it's being dragged
            continue;
        }

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(slot.x as i16, slot.y as i16, card);

        for dc in &mut draw_calls {
            let mut fg_hsl: Hsl = dc.rich_text.fg.into();
            let mut bg_hsl: Hsl = dc.rich_text.bg.into();

            fg_hsl.l *= 0.85;
            bg_hsl.l *= 0.85;

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();
        }

        draw_queue.extend(draw_calls)
    }
}

pub fn draw_table_card_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16) {
    for slot_index in 0..TABLE_SLOT_COUNT {
        let local_x: u16 = x + slot_index * TABLE_CARD_X_SPACING;
        // let locked: bool = !matches!(ctx.slots.state, SlotsState::Idle);
        let locked: bool = false;

        let bg_color: Rgba = if locked {
            let light_red: Rgba = Rgba::from_u8(90, 0, 0, CARD_SLOT_COLOR.a);
            CARD_SLOT_COLOR.lerp(light_red, 0.5)
        } else {
            CARD_SLOT_COLOR
        };

        draw_rect(
            draw_queue,
            local_x as i16,
            y as i16,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
            bg_color,
        );

        if locked {
            draw_queue.push(DrawCall {
                x: local_x + 1,
                y: y + 1,
                rich_text: RichText::new("X")
                    .with_fg(Rgba::from_u8(153, 30, 30, 1.0))
                    .with_bold(true),
            });
        }
    }
}
