use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEventKind};

use crate::{
    button::{Button, get_button_at},
    constants::{
        HAND_CARD_X_SPACING, HAND_ORIGIN_X, HAND_ORIGIN_Y, TABLE_CARD_X_SPACING, TABLE_ORIGIN_X,
        TABLE_ORIGIN_Y, TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    },
    context::Context,
    dragged_card::{
        CardDragState, DragAndDropLocation, delete_card_at, get_valid_drop_destination,
        location_has_card, place_card_at, swap_cards_at,
    },
    playing_card::get_card_hitbox_rect,
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
        Event::Key(KeyEvent { code: key_code, .. }) => match key_code {
            KeyCode::Char('q') => return ProgramStatus::Exit,
            KeyCode::Char('v') => ctx.vignette = !ctx.vignette,
            KeyCode::Char('g') => ctx.gamma_correction = !ctx.gamma_correction,
            _ => {}
        },
        Event::Mouse(mouse_event) => match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => on_left_click_down(ctx),
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
    // > Hand
    for (index, card_in_hand) in iter_some(&ctx.hand.cards_in_hand) {
        let (x1, y1, x2, y2) =
            get_card_hitbox_rect(HAND_ORIGIN_X, HAND_ORIGIN_Y, HAND_CARD_X_SPACING, index);
        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: card_in_hand.card.clone(),
                source: DragAndDropLocation::Hand { index },
            };
        }
    }
    // > Table
    for (index, card_on_table) in iter_some(&ctx.table.cards_on_table) {
        let (x1, y1, x2, y2) =
            get_card_hitbox_rect(TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_CARD_X_SPACING, index);
        if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
            ctx.mouse.card_drag = CardDragState::Dragging {
                card: card_on_table.card.clone(),
                source: DragAndDropLocation::Table { index: index },
            };
        }
    }
}

fn on_left_click_up(ctx: &mut Context, buttons: &[Button]) {
    ctx.mouse.is_left_down = false;

    // Run click callbacks on buttons if not dragging
    if matches!(ctx.mouse.card_drag, CardDragState::NotDragging) {
        if let Some(button) = get_button_at(&buttons, ctx.mouse.x, ctx.mouse.y) {
            (button.on_click)(ctx);
        }
    }

    // Drop logic
    let drag_state = std::mem::replace(&mut ctx.mouse.card_drag, CardDragState::NotDragging);

    if let CardDragState::Dragging { card, source } = drag_state {
        if let Some(destination) = get_valid_drop_destination(ctx, &source) {
            if location_has_card(ctx, &destination) {
                swap_cards_at(ctx, &source, &destination);
            } else {
                place_card_at(ctx, card, &destination);
                delete_card_at(ctx, &source);
            }
        }
    }
}
