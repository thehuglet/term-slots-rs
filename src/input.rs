use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind};

use crate::{
    TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    button::{Button, get_button_at},
    card::{BIG_CARD_HEIGHT, BIG_CARD_WIDTH, Card},
    card_ops::{
        CardDragAndDropLocation, CardDragState, delete_card_at, get_valid_drop_destination,
        location_has_card, place_card_at, swap_cards,
    },
    context::Context,
    hand::{HAND_CARD_X_SPACING, HAND_ORIGIN_X, HAND_ORIGIN_Y, HAND_SLOT_COUNT},
    poker_hand::update_current_poker_hand,
    renderer::{Screen, point_in_rect},
    table::{TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_SLOT_COUNT},
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
            MouseEventKind::Down(MouseButton::Right) => on_right_click_down(ctx, buttons),
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
    let table_slots_with_cards = ctx
        .table_card_slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.card.as_ref().map(|card| (index, slot, card)));

    for (index, _, card) in table_slots_with_cards {
        let x1: u16 = TABLE_ORIGIN_X + index as u16 * TABLE_CARD_X_SPACING;
        let y1: u16 = TABLE_ORIGIN_Y;

        if point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
        ) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: *card,
                source: CardDragAndDropLocation::Table { index },
            };
        }
    }

    // > Hand
    let hand_slots_with_cards = ctx
        .hand_card_slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.card.as_ref().map(|card| (index, slot, card)));

    for (index, _, card) in hand_slots_with_cards {
        let x1: u16 = HAND_ORIGIN_X + index as u16 * HAND_CARD_X_SPACING;
        let y1: u16 = HAND_ORIGIN_Y;

        if point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
        ) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: *card,
                source: CardDragAndDropLocation::Hand { index },
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
    let maybe_drag_data: Option<(crate::card::Card, CardDragAndDropLocation)> = match drag_state {
        CardDragState::Dragging { card, source } => Some((card, source)),
        _ => None,
    };

    if let Some((card, source)) = maybe_drag_data
        && let Some(destination) = get_valid_drop_destination(ctx, &source)
    {
        if location_has_card(ctx, &destination) {
            swap_cards(ctx, &source, &destination);
        } else {
            place_card_at(ctx, card, &destination);
            delete_card_at(ctx, &source);
        }
    }
}

fn on_right_click_down(ctx: &mut Context, buttons: &[Button]) {
    // Fast button clicks (not enabled on all)
    let maybe_button: Option<&Button> = get_button_at(buttons, ctx.mouse.x, ctx.mouse.y);
    if let Some(button) = maybe_button
        && button.allow_rmb
        && (button.enabled_when)(ctx)
    {
        (button.on_click)(ctx);
    }

    // Check if clicked on a table card first (Table -> Hand)
    for table_slot_index in 0..TABLE_SLOT_COUNT {
        let x1: u16 = TABLE_ORIGIN_X + table_slot_index * HAND_CARD_X_SPACING;
        let y1: u16 = TABLE_ORIGIN_Y;

        let hitbox_not_clicked: bool = !point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
        );
        let source_slot_empty: bool = ctx.table_card_slots[table_slot_index as usize]
            .card
            .is_none();

        if hitbox_not_clicked || source_slot_empty {
            continue;
        }

        let hand_empty_slots: usize = ctx
            .hand_card_slots
            .iter()
            .filter(|slot| slot.card.is_none())
            .count();

        if hand_empty_slots == 0 {
            return; // Hand is full, can't take table card
        }

        if let Some(empty_hand_slot) = ctx
            .hand_card_slots
            .iter_mut()
            .find(|slot| slot.card.is_none())
        {
            let card: Card = ctx.table_card_slots[table_slot_index as usize]
                .card
                .take()
                .expect("We already checked this exists");

            empty_hand_slot.card = Some(card);

            update_current_poker_hand(ctx);

            // Table card moved to hand, can skip rest of loop
            return;
        }
    }

    // Check if clicked on a hand card (Hand -> Table)
    for hand_slot_index in 0..HAND_SLOT_COUNT {
        let x1: u16 = HAND_ORIGIN_X + hand_slot_index * HAND_CARD_X_SPACING;
        let y1: u16 = HAND_ORIGIN_Y;

        let hitbox_not_clicked: bool = !point_in_rect(
            ctx.mouse.x,
            ctx.mouse.y,
            x1,
            y1,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
        );
        let source_slot_empty: bool = ctx.hand_card_slots[hand_slot_index as usize].card.is_none();

        if hitbox_not_clicked || source_slot_empty {
            continue;
        }

        let table_empty_slots: usize = ctx
            .table_card_slots
            .iter()
            .filter(|slot| slot.card.is_none())
            .count();

        if table_empty_slots == 0 {
            return; // Table is full, can't take hand card
        }

        if let Some(empty_table_slot) = ctx
            .table_card_slots
            .iter_mut()
            .find(|slot| slot.card.is_none())
        {
            let card = ctx.hand_card_slots[hand_slot_index as usize]
                .card
                .take()
                .expect("We already checked this exists");

            empty_table_slot.card = Some(card);

            update_current_poker_hand(ctx);

            // Hand card moved to table, can skip rest of loop
            return;
        }
    }
    update_current_poker_hand(ctx);
}
