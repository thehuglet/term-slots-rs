use crate::{
    card::{Card, draw_calls_playing_card_big},
    card_ops::{CardDragAndDropLocation, CardDragState},
    constants::{
        BIG_PLAYING_CARD_HEIGHT, BIG_PLAYING_CARD_WIDTH, CARD_SLOT_COLOR, HAND_CARD_X_SPACING,
        HAND_SLOT_COUNT,
    },
    context::Context,
    renderer::{DrawCall, Hsl, draw_rect},
    utils::iter_some,
};

pub fn draw_hand(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, ctx: &Context) {
    let slots_with_cards = ctx
        .hand_card_slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| match &slot.card {
            Some(card) => Some((index, slot, card)),
            None => None,
        });

    for (index, slot, card) in slots_with_cards {
        let card_is_being_dragged = matches!(ctx.mouse.card_drag,
            CardDragState::Dragging {
                source: CardDragAndDropLocation::Hand { index: src_index },
                ..
            } if src_index == index
        );

        if card_is_being_dragged {
            // Skip drawing the card at the slot
            // while it's being dragged
            continue;
        }

        let mut draw_calls = draw_calls_playing_card_big(slot.x as i16, slot.y as i16 as i16, card);

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

pub fn draw_hand_card_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16) {
    for n in 0..HAND_SLOT_COUNT {
        draw_rect(
            draw_queue,
            (x + n * HAND_CARD_X_SPACING) as i16,
            y as i16,
            BIG_PLAYING_CARD_WIDTH,
            BIG_PLAYING_CARD_HEIGHT,
            CARD_SLOT_COLOR,
        );
    }
}
