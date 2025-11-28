use crate::{
    playing_card::{PlayingCard, draw_calls_playing_card_big},
    renderer::{DrawCall, HSL},
};

pub const HAND_CARD_X_SPACING: u16 = 4;

pub struct Hand {
    pub cursor: u8,
    pub hand_size: u8,
    pub cards_in_hand: Vec<CardInHand>,
}

#[derive(Clone)]
pub struct CardInHand {
    pub card: PlayingCard,
}

pub fn draw_hand(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, hand: &Hand) {
    for (card_in_hand_index, card_in_hand) in hand.cards_in_hand.iter().enumerate() {
        let card: &PlayingCard = &card_in_hand.card;

        let n: u16 = card_in_hand_index as u16;
        let card_x: u16 = x + n * HAND_CARD_X_SPACING;
        let card_y: u16 = y;

        let mut draw_calls: Vec<DrawCall> = draw_calls_playing_card_big(card_x, card_y, card);

        for dc in &mut draw_calls {
            let mut fg_hsl: HSL = dc.rich_text.fg.into();
            let mut bg_hsl: HSL = dc.rich_text.bg.into();

            fg_hsl.l *= 0.7;
            bg_hsl.l *= 0.7;

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();
        }

        draw_queue.extend(draw_calls)
    }
}
