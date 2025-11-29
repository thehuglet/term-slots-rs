use crate::playing_card::PlayingCard;

pub struct DraggedCardContext {
    pub card: Option<PlayingCard>,
    pub dragged_card: Option<PlayingCard>,
}
