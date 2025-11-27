use crate::{
    playing_card::{PlayingCard, draw_calls_playing_card_small},
    renderer::{DrawCall, HSL, RGBA},
};

pub struct Slots {
    pub spin_count: u32,
    pub columns: Vec<Column>,
}

#[derive(Clone)]
pub struct Column {
    pub cursor: f32,
    pub cards: Vec<PlayingCard>,
    pub spin_duration: f32,
    pub spin_time_remaining: f32,
    pub spin_speed: f32,
}

pub fn draw_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, slots: &Slots) {
    const X_SPACING: u16 = 10;

    for (col_index, column) in slots.columns.iter().enumerate() {
        let n: u16 = col_index as u16;
        let column_x: u16 = x + n * X_SPACING;
        let column_y: u16 = y;

        draw_column(draw_queue, column_x, column_y, column);
    }
}

fn draw_column(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, column: &Column) {
    fn get_card_index(row_offset: i16, column: &Column) -> usize {
        let len: i16 = column.cards.len() as i16;
        let index: i16 = column.cursor as i16 + row_offset;
        let wrapped_index: i16 = index.rem_euclid(len);
        wrapped_index as usize
    }

    for row_offset in -3..4 as i16 {
        let card_index: usize = get_card_index(row_offset, column);
        let card: &PlayingCard = &column.cards[card_index];

        // Y is casted to ensure the subtraction is done on a signed int
        // as card_y is never supposed to be in the negatives
        let card_y_signed: i16 = y as i16 + row_offset;
        debug_assert!(
            card_y_signed >= 0,
            "Y position would be negative: {} + {} = {}",
            y,
            row_offset,
            card_y_signed
        );

        let card_x: u16 = x;
        let card_y: u16 = card_y_signed as u16;

        let mut card_draw_call: DrawCall = draw_calls_playing_card_small(card_x, card_y, card);

        if row_offset != 0 {
            let mut fg_hsl = HSL::from_rgba(card_draw_call.rich_text.fg);
            let mut bg_hsl = HSL::from_rgba(card_draw_call.rich_text.bg);

            let sigma: f32 = 1.5;
            let gaussian_factor: f32 = (-(row_offset.pow(2) as f32) / (2.0 * sigma.powi(2))).exp();
            fg_hsl.l *= gaussian_factor * 0.7;
            bg_hsl.l *= gaussian_factor * 0.7;

            card_draw_call.rich_text.fg = RGBA::from_hsl(fg_hsl);
            card_draw_call.rich_text.bg = RGBA::from_hsl(bg_hsl);
        }

        draw_queue.push(card_draw_call);
    }
}
