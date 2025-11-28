use std::process::exit;

use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEventKind};

use crate::{
    button::Button,
    constants::{HAND_ORIGIN_X, HAND_ORIGIN_Y},
    context::Context,
    hand::HAND_CARD_X_SPACING,
    playing_card::{BIG_CARD_HEIGHT, BIG_CARD_WIDTH},
    renderer::{Screen, point_in_rect},
};

#[derive(PartialEq)]
pub enum ProgramStatus {
    Running,
    Exit,
}

pub fn resolve_input(ctx: &mut Context, event: Event, buttons: &[Button]) -> ProgramStatus {
    match event {
        Event::Resize(w, h) => {
            // Recreate screen on resize to avoid graphical anomalies
            ctx.screen = Screen::new(w, h, (0, 0, 0));
        }
        Event::Key(KeyEvent { code, .. }) => match code {
            KeyCode::Char('q') | KeyCode::Esc => return ProgramStatus::Exit,
            _ => {}
        },
        Event::Mouse(event) => match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                ctx.mouse.is_down = true;
            }
            MouseEventKind::Up(MouseButton::Left) => {
                ctx.mouse.is_down = false;

                // Card ui rect detection
                // > Hand
                for card_index in 0..ctx.hand.cards_in_hand.len() {
                    let x1: u16 = HAND_ORIGIN_X + card_index as u16 * HAND_CARD_X_SPACING;
                    let y1: u16 = HAND_ORIGIN_Y;
                    let x2: u16 = x1 + BIG_CARD_WIDTH - 1;
                    let y2: u16 = y1 + BIG_CARD_HEIGHT - 1;
                    if point_in_rect(ctx.mouse.x, ctx.mouse.y, x1, y1, x2, y2) {
                        exit(0)
                    }
                }

                for button in buttons {
                    let button_x2: u16 = button.x + button.text.len() as u16 + 1;
                    let button_y2: u16 = button.y;

                    if point_in_rect(
                        ctx.mouse.x,
                        ctx.mouse.y,
                        button.x,
                        button.y,
                        button_x2,
                        button_y2,
                    ) {
                        if (button.enabled_when)(ctx) {
                            (button.on_click)(ctx);
                        }
                    };
                }
            }
            MouseEventKind::Moved => {
                ctx.mouse.x = event.column;
                ctx.mouse.y = event.row;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                ctx.mouse.x = event.column;
                ctx.mouse.y = event.row;
            }
            _ => {}
        },
        _ => {}
    }
    ProgramStatus::Running
}

// pub fn resolve_action(ctx: &mut Context, action: Option<Action>) -> ProgramStatus {
//     if let Some(a) = action {
//         match a {
//             Action::ExitGame => return ProgramStatus::Exit,
//             Action::SpinSlots => {}
//             Action::MouseMove { x, y } => {
//                 ctx.mouse.x = x;
//                 ctx.mouse.y = y;
//             }
//             Action::WindowResize { w, h } => {
//                 // Recreate screen on resize to avoid graphical anomalies
//                 ctx.screen = Screen::new(w, h, RGBA::from_u8(0, 0, 0, 1.0));
//             }
//             Action::MouseDrag => {}
//         }
//     }

//     ProgramStatus::Running
// }
