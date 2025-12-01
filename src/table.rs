use crate::{
    constants::TABLE_CARD_X_SPACING,
    context::Context,
    dragged_card::{CardDragState, DragAndDropLocation, dragged_source_vfx_sinewave},
    playing_card::{PlayingCard, draw_calls_playing_card_big},
    renderer::{DrawCall, Hsl},
    utils::iter_some,
};

pub struct Table {
    pub cards_on_table: Vec<Option<CardOnTable>>,
}

#[derive(Clone)]
pub struct CardOnTable {
    pub card: PlayingCard,
}

pub fn draw_table(draw_queue: &mut Vec<DrawCall>, ctx: &Context, x: u16, y: u16) {
    for (index, card_on_table) in iter_some(&ctx.table.cards_on_table) {
        let card: &PlayingCard = &card_on_table.card;

        let n: u16 = index as u16;
        let card_x: u16 = x + n * TABLE_CARD_X_SPACING;
        let card_y: u16 = y;
        let is_being_dragged = matches!(ctx.mouse.card_drag,
            CardDragState::Dragging {
                source: DragAndDropLocation::Table { index: src_index },
                ..
            } if src_index == index
        );

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(card_x as i16, card_y as i16, card);

        for dc in &mut draw_calls {
            let mut fg_hsl: Hsl = dc.rich_text.fg.into();
            let mut bg_hsl: Hsl = dc.rich_text.bg.into();

            if is_being_dragged {
                let sinewave: f32 = dragged_source_vfx_sinewave(ctx.game_time);
                fg_hsl.l *= sinewave;
                bg_hsl.l *= sinewave;
            }

            fg_hsl.l *= 0.85;
            bg_hsl.l *= 0.85;

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();
        }

        draw_queue.extend(draw_calls)
    }
}
