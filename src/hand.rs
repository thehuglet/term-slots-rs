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
    for (index, card) in iter_some(&ctx.cards_in_hand) {
        let n: u16 = index as u16;
        let card_x: u16 = x + n * HAND_CARD_X_SPACING;
        let card_y: u16 = y;
        let is_being_dragged = matches!(ctx.mouse.card_drag,
            CardDragState::Dragging {
                source: CardDragAndDropLocation::Hand { index: src_index },
                ..
            } if src_index == index
        );

        if is_being_dragged {
            // Don't draw the card at it's original pos while it's being dragged
            continue;
        }

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(card_x as i16, card_y as i16, card);

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
