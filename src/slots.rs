use crate::{
    playing_card::{PlayingCard, draw_calls_playing_card_small},
    renderer::{DrawCall, HSL, RGBA, RichText},
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

pub fn calc_column_spin_duration_sec(col_index: usize) -> f32 {
    // Total base duration in seconds
    const BASE_SPIN_DURATION_SEC: f32 = 3.0;
    // How much each column adds relative to base
    const STAGGER_RATIO: f32 = 0.35;
    // Minimum time between column stops
    const MIN_STAGGER_DELTA_SEC: f32 = 0.5;
    // Blend between geometric vs linear progression
    const GEOMETRIC_WEIGHT: f32 = 0.3;

    let per_column_base: f32 = BASE_SPIN_DURATION_SEC * STAGGER_RATIO;

    if col_index == 0 {
        return BASE_SPIN_DURATION_SEC;
    }

    // Geometric progression: each column waits less than the previous
    let geometric_staggers: Vec<f32> = (0..col_index)
        .map(|i| (per_column_base * STAGGER_RATIO.powi(i as i32)).max(MIN_STAGGER_DELTA_SEC))
        .collect();

    // Linear progression: each column waits the same amount
    let linear_staggers: Vec<f32> = (0..col_index)
        .map(|_| per_column_base.max(MIN_STAGGER_DELTA_SEC))
        .collect();

    // Blend geometric and linear progressions
    let total_stagger: f32 = (0..col_index)
        .map(|i| {
            linear_staggers[i] * (1.0 - GEOMETRIC_WEIGHT) + geometric_staggers[i] * GEOMETRIC_WEIGHT
        })
        .sum();

    BASE_SPIN_DURATION_SEC + total_stagger
}

pub fn spin_slots_column(column: &mut Column, dt: f32, max_spin_speed: f32) {
    const SNAP_THRESHOLD: f32 = 0.15;

    column.spin_time_remaining = (column.spin_time_remaining - dt).max(0.0);

    // NaN safety
    if column.spin_duration <= 0.0 {
        column.spin_speed = 0.0;
    } else {
        let spin_speed = {
            let exponent = 6.0;
            let time_normalized =
                (column.spin_time_remaining / column.spin_duration).clamp(0.0, 1.0);

            if time_normalized <= 0.0 || time_normalized <= SNAP_THRESHOLD {
                0.0
            } else {
                max_spin_speed * (1.0 - (1.0 - time_normalized).powf(exponent))
            }
        };
        column.spin_speed = spin_speed;
    }

    column.cursor -= column.spin_speed * dt;

    let spin_stopped: bool = column.spin_speed == 0.0;
    if spin_stopped {
        column.spin_time_remaining = 0.0;
    }
}

pub fn draw_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, slots: &Slots) {
    const X_SPACING: u16 = 5;

    for (col_index, column) in slots.columns.iter().enumerate() {
        let n: u16 = col_index as u16;
        let column_x: u16 = x + n * X_SPACING;
        let column_y: u16 = y;

        draw_column(draw_queue, column_x, column_y, column);
    }
}

fn draw_column(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, column: &Column) {
    fn get_card_index(row_offset: i16, column: &Column) -> usize {
        let cards_len: i16 = column.cards.len() as i16;
        let index: i16 = column.cursor as i16 + row_offset;
        let wrapped_index: i16 = index.rem_euclid(cards_len);
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
            "Slots Y position would be negative: {} + {} = {}",
            y,
            row_offset,
            card_y_signed
        );

        let card_x: u16 = x;
        let card_y: u16 = card_y_signed as u16;

        let mut card_draw_call: DrawCall = draw_calls_playing_card_small(card_x, card_y, card);

        if row_offset != 0 {
            let mut fg_hsl: HSL = card_draw_call.rich_text.fg.into();
            let mut bg_hsl: HSL = card_draw_call.rich_text.bg.into();

            let sigma: f32 = 1.5;
            let gaussian_factor: f32 = (-(row_offset.pow(2) as f32) / (2.0 * sigma.powi(2))).exp();
            fg_hsl.l *= gaussian_factor * 0.7;
            bg_hsl.l *= gaussian_factor * 0.7;

            card_draw_call.rich_text.fg = fg_hsl.into();
            card_draw_call.rich_text.bg = bg_hsl.into();
        }

        draw_queue.push(card_draw_call);
    }
}

pub fn slots_stopped(slots: &Slots) -> bool {
    slots.columns.iter().all(|column| column.spin_speed == 0.0)
}
