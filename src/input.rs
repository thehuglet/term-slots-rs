use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEventKind};

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

#[derive(Clone, Copy)]
pub enum Action {
    ExitGame,
    SpinSlots,
    MouseMoved { x: u16, y: u16 },
    WindowResize { w: u16, h: u16 },
}

pub fn to_action(ctx: &mut Context, event: Event, buttons: &[Button]) -> Option<Action> {
    match event {
        Event::Key(KeyEvent { code, .. }) => match code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::ExitGame),
            _ => None,
        },
        Event::Mouse(event) => match event.kind {
            MouseEventKind::Up(_) => {
                let (mx, my) = (event.column as usize, event.row as usize);

                for b in buttons {
                    let b_x2: usize = b.x + b.text.len() + 1;
                    let b_y2: usize = b.y;

                    if point_in_rect(mx, my, b.x, b.y, b_x2, b_y2) {
                        return Some(b.action);
                    };
                }

                None
            }
            MouseEventKind::Moved => {
                return Some(Action::MouseMoved {
                    x: event.column,
                    y: event.row,
                });
            }
            _ => None,
        },
        Event::Resize(w, h) => Some(Action::WindowResize { w, h }),
        _ => None,
    }
}

pub fn resolve_action(ctx: &mut Context, action: Option<Action>) -> ProgramStatus {
    if let Some(a) = action {
        match a {
            Action::ExitGame => return ProgramStatus::Exit,
            Action::SpinSlots => {}
            Action::MouseMoved { x, y } => ctx.mouse_pos = (x, y),
            Action::WindowResize { w, h } => {
                // Recreate screen on resize to avoid graphical anomalies
                ctx.screen = Screen::new(w as usize, h as usize, RGBA::from_u8(0, 0, 0, 1.0));
            }
        }
    }

    ProgramStatus::Running
}
