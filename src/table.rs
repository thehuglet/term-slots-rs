use crate::{
    constants::{
        BIG_PLAYING_CARD_HEIGHT, BIG_PLAYING_CARD_WIDTH, CARD_SLOT_COLOR, TABLE_CARD_X_SPACING,
        TABLE_SLOT_COUNT,
    },
    context::Context,
    dragged_card::{CardDragState, DragAndDropLocation},
    playing_card::{PlayingCard, draw_calls_playing_card_big},
    poker_hand::PokerHand,
    renderer::{DrawCall, Hsl, Rgba, RichText, draw_rect},
    slots::SlotsState,
    utils::iter_some,
};

pub struct Table {
    pub poker_hand: Option<PokerHand>,
    pub cards_on_table: Vec<Option<CardOnTable>>,
}

#[derive(Clone)]
pub struct CardOnTable {
    pub card: PlayingCard,
}

pub fn draw_table(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, ctx: &Context) {
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

        if is_being_dragged {
            // Don't draw the card at it's original pos while it's being dragged
            continue;
        }

        let mut draw_calls: Vec<DrawCall> =
            draw_calls_playing_card_big(card_x as i16, card_y as i16, card);

        for dc in &mut draw_calls {
            let mut fg_hsl: Hsl = dc.rich_text.fg.into();
            let mut bg_hsl: Hsl = dc.rich_text.bg.into();

            fg_hsl.l *= 0.85;
            bg_hsl.l *= 0.85;

            dc.rich_text.fg = fg_hsl.into();
            dc.rich_text.bg = bg_hsl.into();
        }

        draw_queue.extend(draw_calls)
    }
}

pub fn draw_table_card_slots(draw_queue: &mut Vec<DrawCall>, x: u16, y: u16, ctx: &Context) {
    for slot_index in 0..TABLE_SLOT_COUNT {
        let local_x: u16 = x + slot_index * TABLE_CARD_X_SPACING;
        // let locked: bool = !matches!(ctx.slots.state, SlotsState::Idle);
        let locked: bool = false;

        let bg_color: Rgba = if locked {
            let light_red: Rgba = Rgba::from_u8(90, 0, 0, CARD_SLOT_COLOR.a);
            CARD_SLOT_COLOR.lerp(light_red, 0.5)
        } else {
            CARD_SLOT_COLOR
        };

        draw_rect(
            draw_queue,
            local_x as i16,
            y as i16,
            BIG_PLAYING_CARD_WIDTH,
            BIG_PLAYING_CARD_HEIGHT,
            bg_color,
        );

        if locked {
            draw_queue.push(DrawCall {
                x: local_x + 1,
                y: y + 1,
                rich_text: RichText::new("X")
                    .with_fg(Rgba::from_u8(153, 30, 30, 1.0))
                    .with_bold(true),
            });
        }
    }
}
