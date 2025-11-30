use crate::{
    constants::{
        HAND_CARD_X_SPACING, HAND_ORIGIN_X, HAND_ORIGIN_Y, TABLE_CARD_X_SPACING, TABLE_ORIGIN_X,
        TABLE_ORIGIN_Y,
    },
    context::Context,
    hand::{CardInHand, Hand},
    playing_card::{PlayingCard, get_card_hitbox_rect},
    renderer::point_in_rect,
    table::{CardOnTable, Table},
};

pub enum CardDragState {
    NotDragging,
    Dragging {
        card: PlayingCard,
        source: DragAndDropLocation,
    },
}

pub enum DragAndDropLocation {
    Hand { index: usize },
    Table { index: usize },
}

pub fn dragged_source_vfx_sinewave(t: f32) -> f32 {
    let frequency: f32 = 7.5;
    0.5 + 0.25 * ((frequency * t).sin() + 1.0)
}

/// Retrieves the location at which a card was dropped if it was dropped on one.
pub fn get_valid_drop_destination(
    ctx: &mut Context,
    source: &DragAndDropLocation,
) -> Option<DragAndDropLocation> {
    for dest_index in 0..ctx.table.cards_on_table.len() {
        // Table checks
        let (x1, y1, x2, y2) = get_card_hitbox_rect(
            TABLE_ORIGIN_X,
            TABLE_ORIGIN_Y,
            TABLE_CARD_X_SPACING,
            dest_index,
        );
        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            let destination = DragAndDropLocation::Table { index: dest_index };

            if let DragAndDropLocation::Table { index: src_index } = source {
                if dest_index == *src_index {
                    continue;
                }
            }

            return Some(destination);
        }
    }

    for dest_index in 0..ctx.hand.cards_in_hand.len() {
        // Hand checks
        let (x1, y1, x2, y2) = get_card_hitbox_rect(
            HAND_ORIGIN_X,
            HAND_ORIGIN_Y,
            HAND_CARD_X_SPACING,
            dest_index,
        );
        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            let destination = DragAndDropLocation::Hand { index: dest_index };

            if let DragAndDropLocation::Hand { index: src_index } = source {
                if dest_index == *src_index {
                    continue;
                }
            }

            return Some(destination);
        }
    }
    None
}

fn try_take_card_from(ctx: &mut Context, location: &DragAndDropLocation) -> Option<PlayingCard> {
    match location {
        DragAndDropLocation::Hand { index } => {
            // Take the card from hand slot, extract the PlayingCard inside
            ctx.hand.cards_in_hand[*index].take().map(|c| c.card)
        }
        DragAndDropLocation::Table { index } => {
            // Take the card from table slot, extract the PlayingCard inside
            ctx.table.cards_on_table[*index].take().map(|c| c.card)
        }
    }
}

pub fn place_card_at(ctx: &mut Context, card: PlayingCard, location: &DragAndDropLocation) {
    match location {
        DragAndDropLocation::Hand { index } => {
            ctx.hand.cards_in_hand[*index] = Some(CardInHand { card: card });
        }
        DragAndDropLocation::Table { index } => {
            ctx.table.cards_on_table[*index] = Some(CardOnTable { card: card });
        }
    }
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
}
