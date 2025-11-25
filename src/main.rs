mod constants;
mod context;
mod fps_counter;
mod fps_limiter;
mod playing_card;
mod renderer;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute, queue,
    style::{Attributes, Color, ContentStyle, Print, ResetColor, SetStyle},
    terminal::{self},
};
use std::{
    io::{self, Stdout, Write},
    time,
};

use crate::{
    context::Context,
    fps_counter::FPSCounter,
    fps_limiter::FPSLimiter,
    playing_card::{PlayingCard, Suit, draw_playing_card_big},
    renderer::{DrawCall, RGBA, RichText, Screen, compose_buffer},
};

/// Return `Result<false>` is as program exit signal.
fn tick(ctx: &mut Context, stdout: &mut Stdout) -> io::Result<bool> {
    // Dirty inline input handling for now
    if event::poll(time::Duration::from_millis(0))? {
        match event::read()? {
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
                    return Ok(false);
                }
            }
            Event::Mouse(mouse_event) => {
                if mouse_event.kind == MouseEventKind::Moved {
                    ctx.mouse_pos = (mouse_event.column, mouse_event.row);
                }
            }
            Event::Resize(w, h) => {
                // Recreate screen on resize to avoid graphical anomalies
                ctx.screen = Screen::new(w as usize, h as usize, RGBA::from_u8(0, 0, 0, 1.0))
            }
            _ => {}
        }
    }

    let mut draw_calls: Vec<DrawCall> = vec![];

    draw_calls.extend(draw_playing_card_big(
        5,
        5,
        &PlayingCard {
            suit: Suit::Spade,
            rank: playing_card::Rank::King,
        },
    ));

    // Experiment
    draw_calls.push(DrawCall {
        x: ctx.mouse_pos.0 as usize,
        y: ctx.mouse_pos.1 as usize,
        text: RichText::new("boop"),
    });

    draw_calls.push(DrawCall {
        x: 0,
        y: 0,
        text: RichText::new(format!("FPS: {:2.2}", ctx.fps_counter.fps())),
    });

    compose_buffer(&mut ctx.screen.new_buffer, &draw_calls);
    let diff = ctx.screen.diff_buffers();

    for (x, y, cell) in diff {
        // TODO: add alpha blending support later
        let fg_color = Color::Rgb {
            r: cell.fg.r,
            g: cell.fg.g,
            b: cell.fg.b,
        };

        let bg_color = Color::Rgb {
            r: cell.bg.r,
            g: cell.bg.g,
            b: cell.bg.b,
        };

        let style: ContentStyle = ContentStyle {
            foreground_color: Some(fg_color),
            background_color: Some(bg_color),
            underline_color: None,
            attributes: Attributes::none(),
        };

        queue!(
            stdout,
            cursor::MoveTo(x as u16, y as u16),
            SetStyle(style),
            Print(cell.ch),
            ResetColor,
        )?;
    }

    stdout.flush()?;
    ctx.screen.swap_buffers();
    Ok(true)
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
        cursor::Hide,
        EnableMouseCapture
    )?;

    let term_size: (u16, u16) = terminal::size()?;

    let mut ctx = Context {
        screen: Screen::new(
            term_size.0 as usize,
            term_size.1 as usize,
            RGBA::from_u8(0, 0, 0, 1.0),
        ),
        mouse_pos: (0, 0),
        game_time: 0.0,
        fps_counter: FPSCounter::new(0.08),
    };

    let mut fps_limiter = FPSLimiter::new(0.0, 0.001, 0.002);

    'game_loop: loop {
        let dt: f64 = fps_limiter.wait();

        if !tick(&mut ctx, &mut stdout)? {
            break 'game_loop;
        }

        ctx.fps_counter.update(dt);
        ctx.game_time += dt;
    }

    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        DisableMouseCapture
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}
