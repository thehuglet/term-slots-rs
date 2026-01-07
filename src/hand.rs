use crate::{
    card::{BIG_CARD_HEIGHT, BIG_CARD_WIDTH, draw_calls_playing_card_big},
    card_ops::{CardDragAndDropLocation, CardDragState},
    constants::CARD_SLOT_COLOR,
    context::{Context, ImpulseId},
    renderer::{DrawCall, Hsl, Rgba, draw_rect},
};

pub const HAND_ORIGIN_X: u16 = 5;
pub const HAND_ORIGIN_Y: u16 = 20;
pub const HAND_CARD_X_SPACING: u16 = 4;
pub const HAND_SLOT_COUNT: u16 = 7;

pub fn draw_hand(draw_queue: &mut Vec<DrawCall>, ctx: &Context) {
    let slots_with_cards = ctx
        .hand_card_slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.card.as_ref().map(|card| (index, slot, card)));

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

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(slot.x as i16, slot.y as i16, card);

        for dc in &mut draw_calls {
            // HSL ops
            let mut fg_hsl: Hsl = dc.rich_text.fg.into();
            let mut bg_hsl: Hsl = dc.rich_text.bg.into();

            fg_hsl.l *= 0.85;
            bg_hsl.l *= 0.85;

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();

            // Out of space hint impulse
            if let Some(timestamp) = ctx.impulse_timestamps.get(&ImpulseId::NoSpaceInHandHint) {
                let duration: f32 = 0.75;
                let age: f32 = ctx.game_time - timestamp;
                let age_normalized: f32 = age / duration;

                let ramp_frac: f32 = 0.05;

                let t: f32 = if age_normalized < ramp_frac {
                    0.5 + 0.5 * age_normalized / ramp_frac
                } else {
                    let decay_t = (age_normalized - ramp_frac) / (1.0 - ramp_frac);
                    1.0 - decay_t.powi(2)
                };

                if t > 0.0 {
                    let impulse_color = Rgba::from_u8(255, 100, 100, 1.0);
                    dc.rich_text.bg = dc.rich_text.bg.lerp(impulse_color, t);
                }
            }
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
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
            CARD_SLOT_COLOR,
        );
    }
}
