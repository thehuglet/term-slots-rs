use crate::{
    card::{Card, draw_calls_playing_card_small},
    constants::{SLOTS_COLUMNS_X_SPACING, SLOTS_MAX_COLUMN_COUNT, SLOTS_NEIGHBOR_ROW_COUNT},
    context::Context,
    dragged_card::CardDragState,
    renderer::{DrawCall, Hsl, Rgba, RichText, draw_rect, point_in_rect},
};

pub enum SlotsState {
    Idle,
    Spinning,
    PostSpin,
}

pub struct Slots {
    pub state: SlotsState,
    pub spin_count: i32,
    pub columns: Vec<Column>,
}

#[derive(Clone)]
pub struct Column {
    pub cursor: f32,
    pub cards: Vec<Card>,
    pub spin_duration: f32,
    pub spin_time_remaining: f32,
    pub spin_speed: f32,
}

pub fn build_spin_cost_lut(max_spins: usize) -> Vec<i32> {
    let base: f32 = 5.0;
    let growth: f32 = 1.3;
    let divisor: f32 = 3.0;

    let mut lut: Vec<i32> = Vec::with_capacity(max_spins);

    for spin in 0..max_spins {
        let cost: f32 = if spin == 0 {
            base
        } else {
            let exp: f32 = spin as f32 / divisor; // spin, not spin-1
            base * growth.powf(exp)
        };
        lut.push(cost.round() as i32);
    }

    lut
}

pub fn spin_cost(spin_count: i32, lut: &[i32]) -> i32 {
    let cost_index: usize = spin_count as usize;
    lut.get(cost_index).copied().unwrap_or_else(|| {
        let last: i32 = *lut.last().unwrap_or(&5);
        let extra: usize = spin_count as usize - (lut.len() - 1);
        (last as f32 * 1.3f32.powf(extra as f32 / 3.0)).round() as i32
    })
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
        let spin_speed: f32 = {
            let exponent: f32 = 6.0;
            let t: f32 = (column.spin_time_remaining / column.spin_duration).clamp(0.0, 1.0);

            if t <= 0.0 || t <= SNAP_THRESHOLD {
                0.0
            } else {
                max_spin_speed * (1.0 - (1.0 - t).powf(exponent))
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

pub fn slots_are_spinning(slots: &Slots) -> bool {
    slots.columns.iter().all(|column| column.spin_speed == 0.0)
}

pub fn get_column_card_index(row_offset: i16, column: &Column) -> usize {
    let cards_len: i16 = column.cards.len() as i16;
    let index: i16 = column.cursor as i16 + row_offset;
    let wrapped_index: i16 = index.rem_euclid(cards_len);
    wrapped_index as usize
}

/// Slot columns are supposed to be drawn on top of this.
pub fn draw_slots_panel(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, w: u16, h: u16) {
    // Midpart
    let midpart_y: u16 = y + 1;
    for slots_row_index in 0..midpart_y + h {
        let center_row = h / 2 + 1;
        // let center_row = h / 2;
        let distance: f32 = (slots_row_index as i16 - center_row as i16).abs() as f32;
        let max_distance: f32 = center_row as f32;

        let scale: f32 = 1.0 - (distance / max_distance) * 0.3;

        let mut bg_hsl: Hsl = Rgba::from_u8(255, 215, 0, 1.0).into();
        bg_hsl.l *= scale;
        bg_hsl.h += -36.0;
        bg_hsl.l *= 0.25;
        bg_hsl.s *= 0.4;

        let fg: Rgba = Rgba::from_u8(0, 0, 0, 1.0);
        let bg: Rgba = bg_hsl.into();

        draw_queue.push(DrawCall {
            x,
            y: y + slots_row_index,
            rich_text: RichText::new(" ".repeat(w.into())).with_fg(fg).with_bg(bg),
        });
    }

    // Top & bottom borders
    let half_width: i16 = (w / 2) as i16;
    for y in [1, 9] {
        for x in 0..w as i16 {
            draw_rect(draw_queue, x, y, 1, 1, {
                let mut hsl: Hsl = Rgba::from_u8(176, 144, 61, 1.0).into();
                let distance_from_center: i16 = (x - half_width).abs();
                hsl.l *= 0.6 + 0.035 * (half_width - distance_from_center) as f32;
                hsl.s *= 0.8;
                hsl.into()
            });
        }
    }

    // Under-panel shadow
    draw_rect(
        draw_queue,
        0,
        (y + h + 2) as i16,
        w,
        1,
        Rgba::from_u8(0, 0, 0, 0.1),
    );
}

pub fn draw_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, slots: &Slots, ctx: &Context) {
    let maybe_hovered_card: Option<(usize, &Card)> =
        ctx.slots
            .columns
            .iter()
            .enumerate()
            .find_map(|(column_index, column)| {
                let column_x: u16 = x + column_index as u16 * SLOTS_COLUMNS_X_SPACING;
                let column_y: u16 = y;

                let is_hovering: bool = point_in_rect(
                    ctx.mouse.x,
                    ctx.mouse.y,
                    column_x,
                    column_y - SLOTS_NEIGHBOR_ROW_COUNT as u16,
                    SLOTS_COLUMNS_X_SPACING,
                    1 + SLOTS_NEIGHBOR_ROW_COUNT as u16 * 2, // center + top neighbors + bottom_neighbors
                );

                if is_hovering {
                    let card_index = get_column_card_index(0, column);
                    column
                        .cards
                        .get(card_index)
                        .map(|card| (column_index, card))
                } else {
                    None
                }
            });

    for (column_index, column) in slots.columns.iter().enumerate() {
        let n: u16 = column_index as u16;
        let column_x: u16 = x + n * SLOTS_COLUMNS_X_SPACING;
        let column_y: u16 = y;

        let is_hovered: bool = match maybe_hovered_card {
            Some(hovered_card) => column_index == hovered_card.0,
            None => false,
        };

        let is_matching_hovered: bool = if let Some(hovered_card) = maybe_hovered_card {
            let hovered_card: &Card = hovered_card.1;
            let matching_indexes: Vec<usize> =
                slots_center_row_indexes_matching_card(hovered_card, ctx);
            matching_indexes.contains(&column_index)
        } else {
            false
        };

        draw_column(
            draw_queue,
            column_x,
            column_y,
            column,
            ctx,
            is_hovered,
            is_matching_hovered,
        );
    }
}

fn draw_column(
    draw_queue: &mut Vec<DrawCall>,
    x: u16,
    y: u16,
    column: &Column,
    ctx: &Context,
    is_hovered: bool,
    is_matching_hovered: bool,
) {
    for row_offset in -SLOTS_NEIGHBOR_ROW_COUNT..SLOTS_NEIGHBOR_ROW_COUNT + 1 {
        let card_index: usize = get_column_card_index(row_offset, column);
        let card: &Card = &column.cards[card_index];

        // If `y` is ever negative, the slots are drawn too high up, in which case that's a developer mistake.
        // `debug_assert!` is fine here as the code should never ship with the described case.
        let card_y_signed: i16 = y as i16 + row_offset;
        debug_assert!(
            card_y_signed >= 0,
            "Slots Y position would be negative: {y} + {row_offset} = {card_y_signed}"
        );

        let card_x: u16 = x;
        let card_y: u16 = card_y_signed as u16;

        let mut card_draw_calls: DrawCall = draw_calls_playing_card_small(card_x, card_y, card);

        if row_offset == 0 {
            if matches!(ctx.slots.state, SlotsState::PostSpin) {
                let interact_with_me_color = Rgba::from_f32(1.0, 1.0, 0.3, 1.0);
                let highlight_color = Rgba::from_f32(0.0, 1.0, 0.0, 1.0);

                // card.rich_text.fg = card.rich_text.fg.lerp(flash_color, t * 0.75);
                card_draw_calls.rich_text.bg = card_draw_calls
                    .rich_text
                    .bg
                    .lerp(interact_with_me_color, 0.9);

                let not_dragging: bool = matches!(ctx.mouse.card_drag, CardDragState::NotDragging);

                if is_hovered && not_dragging {
                    // Hovered card highlighting
                    card_draw_calls.rich_text.fg =
                        card_draw_calls.rich_text.fg.lerp(highlight_color, 0.2);
                    card_draw_calls.rich_text.bg =
                        card_draw_calls.rich_text.bg.lerp(highlight_color, 1.0);
                }

                if is_matching_hovered {
                    // Matching card highlighting
                    card_draw_calls.rich_text.fg =
                        card_draw_calls.rich_text.fg.lerp(highlight_color, 0.2);
                    card_draw_calls.rich_text.bg =
                        card_draw_calls.rich_text.bg.lerp(highlight_color, 1.0);
                }

                // Matching cards highlighting
                // let matching_column_indexes: Vec<usize> =
                // slots_center_row_indexes_matching_card(card, ctx);
                // card_draw_calls.rich_text.fg =
                //     card_draw_calls.rich_text.fg.lerp(highlight_color, 0.5);
                // card_draw_calls.rich_text.bg =
                //     card_draw_calls.rich_text.bg.lerp(highlight_color, 0.8);
                // }
            }
        } else {
            let sigma: f32 = 1.5;
            let gaussian_factor: f32 = (-(row_offset.pow(2) as f32) / (2.0 * sigma.powi(2))).exp();
            let black = Rgba::from_u8(0, 0, 0, 1.0);
            card_draw_calls.rich_text.fg = card_draw_calls
                .rich_text
                .fg
                .lerp(black, 1.0 - gaussian_factor);
            card_draw_calls.rich_text.bg = card_draw_calls
                .rich_text
                .bg
                .lerp(black, 1.0 - gaussian_factor);
        }

        draw_queue.push(card_draw_calls);
    }
}

pub fn draw_slots_column_shadows(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16) {
    for column_index in 0..SLOTS_MAX_COLUMN_COUNT {
        let x: i16 = (x + column_index * SLOTS_COLUMNS_X_SPACING) as i16;
        let y: i16 = (y - 3) as i16;
        let shadow_color: Rgba = Rgba::from_u8(0, 0, 0, 0.1);

        draw_rect(draw_queue, x + 2, y, 1, 6, shadow_color)
    }
}

pub fn slots_center_row_indexes_matching_card(target_card: &Card, ctx: &Context) -> Vec<usize> {
    ctx.slots
        .columns
        .iter()
        .enumerate()
        .filter_map(|(col_idx, column)| {
            let center_card_index: usize = get_column_card_index(0, column);
            let card: &Card = &column.cards[center_card_index];

            let ranks_match: bool = card.rank == target_card.rank;
            // let suits_match: bool = card.suit == target_card.suit;

            if ranks_match { Some(col_idx) } else { None }
        })
        .collect()
}
