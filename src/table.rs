use crate::{
    playing_card::{PlayingCard, draw_calls_playing_card_big},
    renderer::{DrawCall, HSL},
};

const TABLE_CARD_X_SPACING: u16 = 4;

pub struct Table {
    pub cards_on_table: Vec<CardOnTable>,
}

#[derive(Clone)]
pub struct CardOnTable {
    pub card: PlayingCard,
}

pub fn draw_table(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, table: &Table) {
    for (card_on_table_index, card_on_table) in table.cards_on_table.iter().enumerate() {
        let card: &PlayingCard = &card_on_table.card;

        let n: u16 = card_on_table_index as u16;
        let card_x: u16 = x + n * TABLE_CARD_X_SPACING;
        let card_y: u16 = y;

        let mut draw_calls: Vec<DrawCall> = draw_calls_playing_card_big(card_x, card_y, card);

        // for dc in &mut draw_calls {
        //     let mut fg_hsl: HSL = dc.rich_text.fg.into();
        //     let mut bg_hsl: HSL = dc.rich_text.bg.into();

        //     fg_hsl.l *= 0.7;
        //     bg_hsl.l *= 0.7;

        //     dc.rich_text.fg = fg_hsl.into();
        //     dc.rich_text.bg = bg_hsl.into();
        // }

        draw_queue.extend(draw_calls)
    }
}
