use crossterm::{
    cursor::MoveLeft,
    event::{Event, KeyCode, KeyEvent, MouseButton, MouseEventKind},
};

use crate::{
    button::Button,
    context::Context,
    renderer::{RGBA, Screen, point_in_rect},
};

#[derive(PartialEq)]
pub enum ProgramStatus {
    Running,
    Exit,
}

// #[derive(Clone, Copy)]
// pub enum Action {
//     ExitGame,
//     SpinSlots,
//     MouseLeftDown,
//     MouseLeftUp,
//     MouseDrag,
//     MouseMove { x: u16, y: u16 },
//     WindowResize { w: u16, h: u16 },
// }

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
            MouseEventKind::Down(MouseButton::Left) => ctx.mouse.is_down = true,
            MouseEventKind::Up(MouseButton::Left) => {
                ctx.mouse.is_down = false
                // ctx.mouse.x = event.column;
                // ctx.mouse.y = event.row;

                // for button in buttons {
                //     let button_x2: u16 = button.x + button.text.len() as u16 + 1;
                //     let button_y2: u16 = button.y;

                //     if point_in_rect(mouse_x, mouse_y, button.x, button.y, button_x2, button_y2) {
                //         return Some(button.action);
                //     };
                // }
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
