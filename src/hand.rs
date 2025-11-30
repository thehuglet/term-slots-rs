use crate::{
    constants::HAND_CARD_X_SPACING,
    context::Context,
    dragged_card::{CardDragState, DragAndDropLocation, dragged_source_vfx_sinewave},
    playing_card::{PlayingCard, draw_calls_playing_card_big},
    renderer::{DrawCall, HSL},
    utils::iter_some,
};

pub struct Hand {
    pub hand_size: u8,
    pub cards_in_hand: Vec<Option<CardInHand>>,
}

#[derive(Clone)]
pub struct CardInHand {
    pub card: PlayingCard,
}

pub fn draw_hand(draw_queue: &mut Vec<DrawCall>, ctx: &Context, x: u16, y: u16) {
    for (index, card_in_hand) in iter_some(&ctx.hand.cards_in_hand) {
        let n: u16 = index as u16;
        let card_x: u16 = x + n * HAND_CARD_X_SPACING;
        let card_y: u16 = y;
        let is_being_dragged = matches!(ctx.mouse.card_drag,
            CardDragState::Dragging {
                source: DragAndDropLocation::Hand { index: src_index },
                ..
            } if src_index == index
        );

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(card_x as i16, card_y as i16, &card_in_hand.card);

        for dc in &mut draw_calls {
            let mut fg_hsl: HSL = dc.rich_text.fg.into();
            let mut bg_hsl: HSL = dc.rich_text.bg.into();

            fg_hsl.l *= 0.85;
            bg_hsl.l *= 0.85;

            if is_being_dragged {
                let sinewave: f32 = dragged_source_vfx_sinewave(ctx.game_time as f32);
                fg_hsl.l *= sinewave;
                bg_hsl.l *= sinewave;
            }

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();
        }

        draw_queue.extend(draw_calls)
    }
}
