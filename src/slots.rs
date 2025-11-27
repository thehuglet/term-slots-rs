use crate::{
    playing_card::{PlayingCard, draw_playing_card_small},
    renderer::DrawCall,
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
    for (col_index, column) in slots.columns.iter().enumerate() {
        let column_x: u16 = y;
        let column_y: u16 = x + col_index as u16;

        draw_column(draw_queue, column_x, column_y, column);
    }
}

fn draw_column(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, column: &Column) {
    fn get_card_index(row_offset: i32, column: &Column) -> usize {
        let len = column.cards.len() as i32;
        let index = column.cursor as i32 + row_offset;
        let wrapped_index = index.rem_euclid(len); // Handles negatives properly
        wrapped_index as usize
    }

    for row_offset in -3..3 {
        let card_index: usize = get_card_index(row_offset, column);

        draw_playing_card_small(draw_queue, x, y, card);
    }
}
