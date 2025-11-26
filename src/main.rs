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
    style::{Print, ResetColor, SetStyle},
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
    playing_card::{PlayingCard, Rank, Suit, draw_playing_card_big, draw_playing_card_small},
    renderer::{
        Cell, DrawCall, RGBA, RichText, Screen, build_crossterm_content_style, compose_buffer,
        diff_buffers, fill_screen_background,
    },
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

    fill_screen_background(&mut ctx.screen.new_buffer, RGBA::from_u8(0, 0, 0, 1.0));
    let mut draw_queue: Vec<DrawCall> = vec![];

    for n in 0..10 {
        draw_playing_card_small(
            &mut draw_queue,
            5 + n * 4,
            5,
            &PlayingCard {
                suit: Suit::Spade,
                rank: Rank::King,
            },
        );
    }

    // Experiment
    draw_queue.push(DrawCall {
        x: ctx.mouse_pos.0.saturating_sub(2) as usize,
        y: ctx.mouse_pos.1.saturating_sub(6) as usize,
        text: RichText::new("Boop!")
            .with_fg(RGBA::from_f32(1.0, 0.0, 0.0, 1.0))
            .with_bold(true),
    });

    draw_queue.push(DrawCall {
        x: 0,
        y: 0,
        text: RichText::new(format!("FPS: {:2.2}", ctx.fps_counter.fps())),
    });

    // --- Rendering boilerplate ---

    compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);
    let diff: Vec<(usize, usize, &Cell)> =
        diff_buffers(&ctx.screen.old_buffer, &ctx.screen.new_buffer);

    for (x, y, cell) in diff {
        queue!(
            stdout,
            cursor::MoveTo(x as u16, y as u16),
            SetStyle(build_crossterm_content_style(&cell)),
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
