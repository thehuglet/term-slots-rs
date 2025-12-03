use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};

use crate::{
    button::{Button, get_button_at},
    card::Card,
    constants::{
        BIG_PLAYING_CARD_HEIGHT, BIG_PLAYING_CARD_WIDTH, HAND_CARD_X_SPACING, HAND_ORIGIN_X,
        HAND_ORIGIN_Y, HAND_SLOT_COUNT, TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y,
        TABLE_SLOT_COUNT, TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    },
    context::Context,
    dragged_card::{
        CardDragState, DragAndDropLocation, delete_card_at, get_valid_drop_destination,
        location_has_card, place_card_at, swap_cards_at,
    },
    poker_hand::update_current_poker_hand,
    renderer::{Screen, point_in_rect},
    utils::iter_some,
};

#[derive(PartialEq)]
pub enum ProgramStatus {
    Running,
    Exit,
}

pub fn resolve_input(ctx: &mut Context, event: Event, buttons: &[Button]) -> ProgramStatus {
    match event {
        Event::Resize(_, _) => {
            // Recreate screen on resize to avoid graphical anomalies
            ctx.screen = Screen::new(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT, (0, 0, 0));
        }
        Event::Key(KeyEvent {
            code: key_code,
            kind: key_kind,
            ..
        }) => {
            if key_kind == KeyEventKind::Press {
                match key_code {
                    KeyCode::Char('q') => return ProgramStatus::Exit,
                    KeyCode::Char('v') => {
                        ctx.settings.vignette_enabled = !ctx.settings.vignette_enabled
                    }
                    KeyCode::Char('b') => {
                        ctx.settings.bg_shader_enabled = !ctx.settings.bg_shader_enabled
                    }
                    _ => {}
                }
            }
        }
        Event::Mouse(mouse_event) => match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => on_left_click_down(ctx),
            MouseEventKind::Down(MouseButton::Right) => on_right_click_down(ctx),
            MouseEventKind::Up(MouseButton::Left) => on_left_click_up(ctx, buttons),
            MouseEventKind::Moved => {
                ctx.mouse.x = mouse_event.column;
                ctx.mouse.y = mouse_event.row;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                ctx.mouse.x = mouse_event.column;
                ctx.mouse.y = mouse_event.row;
            }
            _ => {}
        },
        _ => {}
    }
    ProgramStatus::Running
}

pub fn drain_input() -> impl Iterator<Item = Event> {
    std::iter::from_fn(|| {
        if event::poll(Duration::from_millis(0)).ok()? {
            event::read().ok()
        } else {
            None
        }
    })
}

fn on_left_click_down(ctx: &mut Context) {
    ctx.mouse.is_left_down = true;

    // Drag detection
    // > Table
    for (index, card) in iter_some(&ctx.table.cards_on_table) {
        let x1: u16 = TABLE_ORIGIN_X + index as u16 * TABLE_CARD_X_SPACING;
        let y1: u16 = TABLE_ORIGIN_Y;
        let x2: u16 = x1 + BIG_PLAYING_CARD_WIDTH - 1;
        let y2: u16 = y1 + BIG_PLAYING_CARD_HEIGHT - 1;

        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: *card,
                source: DragAndDropLocation::Table { index },
            };
        }
    }
    // > Hand
    for (index, card) in iter_some(&ctx.hand.cards_in_hand) {
        let x1: u16 = HAND_ORIGIN_X + index as u16 * HAND_CARD_X_SPACING;
        let y1: u16 = HAND_ORIGIN_Y;
        let x2: u16 = x1 + BIG_PLAYING_CARD_WIDTH - 1;
        let y2: u16 = y1 + BIG_PLAYING_CARD_HEIGHT - 1;

        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: *card,
                source: DragAndDropLocation::Hand { index },
            };
        }
    }
}

fn on_left_click_up(ctx: &mut Context, buttons: &[Button]) {
    ctx.mouse.is_left_down = false;

    // Button clicks
    let drag_state: CardDragState =
        std::mem::replace(&mut ctx.mouse.card_drag, CardDragState::NotDragging);
    let not_dragging: bool = matches!(drag_state, CardDragState::NotDragging);
    let maybe_button: Option<&Button> = get_button_at(buttons, ctx.mouse.x, ctx.mouse.y);

    if not_dragging
        && let Some(button) = maybe_button
        && (button.enabled_when)(ctx)
    {
        (button.on_click)(ctx);
    }

    // Dropping (drag & drop)
    let maybe_drag_data: Option<(crate::card::Card, DragAndDropLocation)> = match drag_state {
        CardDragState::Dragging { card, source } => Some((card, source)),
        _ => None,
    };

    if let Some((card, source)) = maybe_drag_data
        && let Some(destination) = get_valid_drop_destination(ctx, &source)
    {
        if location_has_card(ctx, &destination) {
            swap_cards_at(ctx, &source, &destination);
        } else {
            place_card_at(ctx, card, &destination);
            delete_card_at(ctx, &source);
        }
    }
}

fn on_right_click_down(ctx: &mut Context) {
    // Check if clicked on a table card first (Table -> Hand)
    for table_slot_index in 0..TABLE_SLOT_COUNT {
        let x1: u16 = TABLE_ORIGIN_X + table_slot_index * HAND_CARD_X_SPACING;
        let y1: u16 = TABLE_ORIGIN_Y;
        let x2: u16 = x1 + BIG_PLAYING_CARD_WIDTH - 1;
        let y2: u16 = y1 + BIG_PLAYING_CARD_HEIGHT - 1;

        let hitbox_not_clicked: bool = !point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2);
        let source_slot_empty: bool = ctx.table.cards_on_table[table_slot_index as usize].is_none();

        if hitbox_not_clicked || source_slot_empty {
            continue;
        }

        let hand_empty_slots: usize = ctx
            .hand
            .cards_in_hand
            .iter()
            .filter(|slot| slot.is_none())
            .count();

        if hand_empty_slots == 0 {
            return; // Hand is full, can't take table card
        }

        if let Some(empty_hand_slot) = ctx
            .hand
            .cards_in_hand
            .iter_mut()
            .find(|slot| slot.is_none())
        {
            let card: Card = ctx.table.cards_on_table[table_slot_index as usize]
                .take()
                .expect("We already checked this exists");

            *empty_hand_slot = Some(card);

            update_current_poker_hand(ctx);

            // Table card moved to hand, can skip rest of loop
            return;
        }
    }

    // Check if clicked on a hand card (Hand -> Table)
    for hand_slot_index in 0..HAND_SLOT_COUNT {
        let x1: u16 = HAND_ORIGIN_X + hand_slot_index * HAND_CARD_X_SPACING;
        let y1: u16 = HAND_ORIGIN_Y;
        let x2: u16 = x1 + BIG_PLAYING_CARD_WIDTH - 1;
        let y2: u16 = y1 + BIG_PLAYING_CARD_HEIGHT - 1;

        let hitbox_not_clicked: bool = !point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2);
        let source_slot_empty: bool = ctx.hand.cards_in_hand[hand_slot_index as usize].is_none();

        if hitbox_not_clicked || source_slot_empty {
            continue;
        }

        let table_empty_slots: usize = ctx
            .table
            .cards_on_table
            .iter()
            .filter(|slot| slot.is_none())
            .count();

        if table_empty_slots == 0 {
            return; // Table is full, can't take hand card
        }

        if let Some(empty_table_slot) = ctx
            .table
            .cards_on_table
            .iter_mut()
            .find(|slot| slot.is_none())
        {
            let card = ctx.hand.cards_in_hand[hand_slot_index as usize]
                .take()
                .expect("We already checked this exists");

            *empty_table_slot = Some(card);

            update_current_poker_hand(ctx);

            // Hand card moved to table, can skip rest of loop
            return;
        }
    }
    update_current_poker_hand(ctx);
}
