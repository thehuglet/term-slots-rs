use crate::{
    card::{Card, draw_calls_playing_card_big},
    constants::{
        BIG_PLAYING_CARD_HEIGHT, BIG_PLAYING_CARD_WIDTH, HAND_CARD_X_SPACING, HAND_ORIGIN_X,
        HAND_ORIGIN_Y, TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y,
    },
    context::Context,
    poker_hand::update_current_poker_hand,
    renderer::{DrawCall, Rgba, draw_rect, point_in_rect},
};

#[derive(Clone)]
pub enum CardDragState {
    NotDragging,
    Dragging {
        card: Card,
        source: DragAndDropLocation,
    },
}

#[derive(Clone)]
pub enum DragAndDropLocation {
    Hand { index: usize },
    Table { index: usize },
}

/// Retrieves the location at which a card was dropped if it was dropped on one.
pub fn get_valid_drop_destination(
    ctx: &mut Context,
    source_location: &DragAndDropLocation,
) -> Option<DragAndDropLocation> {
    for table_slot_index in 0..ctx.table.cards_on_table.len() {
        // Table checks
        let x1: u16 = TABLE_ORIGIN_X + table_slot_index as u16 * TABLE_CARD_X_SPACING;
        let y1: u16 = TABLE_ORIGIN_Y;

        let destination_is_source: bool = matches!(source_location, DragAndDropLocation::Table { index } if *index == table_slot_index);
        // let destination_is_locked: bool = !matches!(ctx.slots.state, SlotsState::Idle);
        let destination_is_locked: bool = false;
        let hitbox_check_failed: bool = !point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_PLAYING_CARD_WIDTH,
            BIG_PLAYING_CARD_HEIGHT,
        );

        if destination_is_locked {
            continue;
        };

        if destination_is_source || destination_is_locked || hitbox_check_failed {
            continue;
        }

        return Some(DragAndDropLocation::Table {
            index: table_slot_index,
        });
    }

    for hand_slot_index in 0..ctx.hand.cards_in_hand.len() {
        // Hand checks
        let x1: u16 = HAND_ORIGIN_X + hand_slot_index as u16 * HAND_CARD_X_SPACING;
        let y1: u16 = HAND_ORIGIN_Y;

        let destination_is_source: bool = matches!(source_location, DragAndDropLocation::Hand { index } if *index == hand_slot_index);
        let hitbox_check_failed: bool = !point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_PLAYING_CARD_WIDTH,
            BIG_PLAYING_CARD_HEIGHT,
        );

        if destination_is_source || hitbox_check_failed {
            continue;
        }

        return Some(DragAndDropLocation::Hand {
            index: hand_slot_index,
        });
    }
    None
}

pub fn place_card_at(ctx: &mut Context, card: Card, location: &DragAndDropLocation) {
    match location {
        DragAndDropLocation::Hand { index } => {
            ctx.hand.cards_in_hand[*index] = Some(card);
        }
        DragAndDropLocation::Table { index } => {
            ctx.table.cards_on_table[*index] = Some(card);
        }
    }
    update_current_poker_hand(ctx);
}

pub fn delete_card_at(ctx: &mut Context, location: &DragAndDropLocation) {
    match location {
        DragAndDropLocation::Hand { index } => {
            ctx.hand.cards_in_hand[*index] = None;
        }
        DragAndDropLocation::Table { index } => {
            ctx.table.cards_on_table[*index] = None;
        }
    }
    update_current_poker_hand(ctx);
}

pub fn location_has_card(ctx: &mut Context, location: &DragAndDropLocation) -> bool {
    match location {
        DragAndDropLocation::Hand { index } => ctx.hand.cards_in_hand[*index].is_some(),
        DragAndDropLocation::Table { index } => ctx.table.cards_on_table[*index].is_some(),
    }
}

/// Will panic if either of the provided locations don't have a corresponding card.
pub fn swap_cards_at(
    ctx: &mut Context,
    location_a: &DragAndDropLocation,
    location_b: &DragAndDropLocation,
) {
    let card_from_location_a =
        try_take_card_from(ctx, location_a).expect("'location_a' was expected to hold a card.");
    let card_from_location_b =
        try_take_card_from(ctx, location_b).expect("'location_b' was expected to hold a card.");

    place_card_at(ctx, card_from_location_a, location_b);
    place_card_at(ctx, card_from_location_b, location_a);
    update_current_poker_hand(ctx);
}

pub fn draw_dragged_card(draw_queue: &mut Vec<DrawCall>, card: &Card, ctx: &mut Context) {
    let anchor_x: i16 = ctx.mouse.x as i16 - 1;
    let anchor_y: i16 = ctx.mouse.y as i16 - 2;

    // Shadow
    draw_rect(
        draw_queue,
        anchor_x - 1,
        anchor_y + 1,
        BIG_PLAYING_CARD_WIDTH,
        BIG_PLAYING_CARD_HEIGHT,
        Rgba::from_f32(0.0, 0.0, 0.0, 0.13),
    );

    // Card
    draw_queue.extend(draw_calls_playing_card_big(anchor_x, anchor_y, card));
    update_current_poker_hand(ctx);
}

fn try_take_card_from(ctx: &mut Context, location: &DragAndDropLocation) -> Option<Card> {
    let cards: Option<Card> = match location {
        DragAndDropLocation::Hand { index } => {
            ctx.hand.cards_in_hand[*index].take().map(|card: Card| card)
        }
        DragAndDropLocation::Table { index } => ctx.table.cards_on_table[*index]
            .take()
            .map(|card: Card| card),
    };

    update_current_poker_hand(ctx);
    cards
}
