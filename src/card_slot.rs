use crate::card::Card;

pub struct CardSlot {
    x: u16,
    y: u16,
    card: Option<Card>,
}
