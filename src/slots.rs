use crate::{
    constants::{SLOTS_COLUMNS_X_SPACING, SLOTS_MAX_COLUMN_COUNT, SLOTS_NEIGHBOR_ROW_COUNT},
    context::Context,
    dragged_card::CardDragState,
    playing_card::{PlayingCard, draw_calls_playing_card_small},
    renderer::{DrawCall, Hsl, Rgba, RichText, draw_rect, point_in_rect},
};

pub enum SlotsState {
    Idle,
    Spinning,
    PostSpin,
}

pub struct Slots {
    pub state: SlotsState,
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

pub fn build_spin_cost_lut(max_spins: usize) -> Vec<u32> {
    let base: f32 = 5.0;
    let growth: f32 = 1.3;
    let divisor: f32 = 3.0;

    let mut lut: Vec<u32> = Vec::with_capacity(max_spins);

    for spin in 0..max_spins {
        let cost: f32 = if spin == 0 {
            base
        } else {
            let exp: f32 = spin as f32 / divisor; // spin, not spin-1
            base * growth.powf(exp)
        };
        lut.push(cost.round() as u32);
    }

    lut
}

pub fn spin_cost(spin_count: u32, lut: &[u32]) -> u32 {
    let cost_index: usize = spin_count as usize;
    lut.get(cost_index).copied().unwrap_or_else(|| {
        let last: u32 = *lut.last().unwrap_or(&5);
        let extra: usize = spin_count as usize - (lut.len() - 1);
        (last as f64 * 1.3f64.powf(extra as f64 / 3.0)).round() as u32
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
    let any_column_hovered: bool = (0..slots.columns.len()).any(|column_index| {
        let local_x: u16 = x + column_index as u16 * SLOTS_COLUMNS_X_SPACING;

        point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            local_x,
            y - SLOTS_NEIGHBOR_ROW_COUNT as u16,
            local_x + 2,
            y + SLOTS_NEIGHBOR_ROW_COUNT as u16,
        )
    });

    for (col_index, column) in slots.columns.iter().enumerate() {
        let n: u16 = col_index as u16;
        let column_x: u16 = x + n * SLOTS_COLUMNS_X_SPACING;
        let column_y: u16 = y;

        draw_column(
            draw_queue,
            column_x,
            column_y,
            column,
            ctx,
            any_column_hovered,
        );
    }
}

fn draw_column(
    draw_queue: &mut Vec<DrawCall>,
    x: u16,
    y: u16,
    column: &Column,
    ctx: &Context,
    any_column_hovered: bool,
) {
    for row_offset in -SLOTS_NEIGHBOR_ROW_COUNT..SLOTS_NEIGHBOR_ROW_COUNT + 1 {
        let card_index: usize = get_column_card_index(row_offset, column);
        let card: &PlayingCard = &column.cards[card_index];

        // If `y` is ever negative, the slots are drawn too high up, in which case that's a developer mistake.
        // `debug_assert!` is fine here as the code should never ship with the described case.
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

        let mut card: DrawCall = draw_calls_playing_card_small(card_x, card_y, card);

        let mouse_hovering_this_column: bool =
            point_in_rect(ctx.mouse.x, ctx.mouse.y, card_x, card_y, card_x + 2, card_y);

        // // "Interact with me" flashing post-spin
        // if !any_column_hovered {
        //     // let peak: f32 = 0.4;
        //     // let frequency: f32 = 7.0;
        //     // let t: f32 = ((frequency * ctx.game_time).sin() * 0.5 + 0.5) * peak;
        //     let t: f32 = 0.5;
        //     let flash_color = Rgba::from_f32(0.5, 0.5, 0.0, 1.0);

        //     card.rich_text.fg = card.rich_text.fg.lerp(flash_color, t * 0.75);
        //     card.rich_text.bg = card.rich_text.bg.lerp(flash_color, t);
        // }

        if row_offset == 0 {
            if matches!(ctx.slots.state, SlotsState::PostSpin) {
                // Mouse hover experiment
                let flash_color = Rgba::from_f32(1.0, 1.0, 0.3, 1.0);

                // card.rich_text.fg = card.rich_text.fg.lerp(flash_color, t * 0.75);
                card.rich_text.bg = card.rich_text.bg.lerp(flash_color, 0.9);

                let not_dragging: bool = matches!(ctx.mouse.card_drag, CardDragState::NotDragging);

                if mouse_hovering_this_column && not_dragging {
                    let flash_color = Rgba::from_f32(0.0, 1.0, 0.0, 1.0);
                    card.rich_text.fg = card.rich_text.fg.lerp(flash_color, 0.5);
                    card.rich_text.bg = card.rich_text.bg.lerp(flash_color, 0.8);
                    // fg_hsl.l *= 0.1;
                    // bg_hsl.l *= 0.1;
                }
            }
        } else {
            let sigma: f32 = 1.5;
            let gaussian_factor: f32 = (-(row_offset.pow(2) as f32) / (2.0 * sigma.powi(2))).exp();
            let black = Rgba::from_u8(0, 0, 0, 1.0);
            card.rich_text.fg = card.rich_text.fg.lerp(black, 1.0 - gaussian_factor);
            card.rich_text.bg = card.rich_text.bg.lerp(black, 1.0 - gaussian_factor);
        }

        draw_queue.push(card);
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
