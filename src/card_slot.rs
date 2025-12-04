use crate::card::Card;

pub struct CardSlot {
    pub x: u16,
    pub y: u16,
    pub card: Option<Card>,
}

/// Builds a specified amount of card slots with specified x spacing.
///
/// This is useful since slots have an `x` & `y`
/// position that needs to be initialized.
pub fn build_card_slots(
    origin_x: u16,
    origin_y: u16,
    x_spacing: u16,
    count: usize,
) -> Vec<CardSlot> {
    (0..count)
        .into_iter()
        .map(|index| CardSlot {
            x: origin_x + index as u16 * x_spacing,
            y: origin_y,
            card: None,
        })
        .collect()
}
