mod button;
mod constants;
mod context;
mod fps_counter;
mod fps_limiter;
mod game_state;
mod input;
mod playing_card;
mod renderer;
mod slots;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute, queue,
    style::{Print, ResetColor, SetStyle},
    terminal::{self},
};
use std::{
    io::{self, Stdout, Write},
    time::{self, Duration},
};

use crate::{
    button::{Button, draw_button},
    context::{Context, MouseContext},
    fps_counter::{FPSCounter, draw_fps_counter},
    fps_limiter::FPSLimiter,
    input::{ProgramStatus, resolve_input},
    playing_card::{
        PlayingCard, Rank, Suit, draw_calls_playing_card_big, draw_calls_playing_card_small,
    },
    renderer::{
        Cell, DrawCall, HSL, RGBA, RichText, Screen, build_crossterm_content_style, compose_buffer,
        diff_buffers, draw_rect, fill_screen_background,
    },
    slots::{Column, Slots, draw_slots},
};

fn plasma_shader(x: u16, y: u16, time: f32) -> RGBA {
    let fx = x as f32 / 80.0;
    let fy = y as f32 / 24.0;

    let value =
        ((fx + time).sin() + (fy + time * 0.7).cos() + ((fx + fy + time * 1.3).sin() * 2.0)).sin();

    let hue = (value * 0.5 + 0.5) * 360.0;
    RGBA::from_hsl(HSL {
        h: hue,
        s: 0.8,
        l: 0.4,
        a: 1.0,
    })
}

fn tick(ctx: &mut Context, stdout: &mut Stdout) -> io::Result<ProgramStatus> {
    // --- Button definitions ---
    let mut buttons: Vec<Button> = vec![];

    buttons.push(Button {
        x: 50,
        y: 10,
        text: "SPIN",
        color: RGBA::from_u8(255, 151, 0, 1.0),
        disabled: false,
    });

    // --- Inputs ---
    let program_status: ProgramStatus = if event::poll(Duration::from_millis(0))? {
        resolve_input(ctx, event::read()?, &buttons)
    } else {
        ProgramStatus::Running
    };

    // --- Rendering ---
    fill_screen_background(&mut ctx.screen.new_buffer, (0, 0, 0));
    let mut draw_queue: Vec<DrawCall> = vec![];

    // --- PLASMA SHADER BACKGROUND ---
    let (width, height) = (ctx.screen.new_buffer.width, ctx.screen.new_buffer.height);
    for y in 0..height {
        for x in 0..width {
            let color = plasma_shader(x, y, ctx.game_time as f32);
            draw_queue.push(DrawCall {
                x,
                y,
                rich_text: RichText::new(" ").with_bg(color),
            });
        }
    }

    draw_slots(&mut draw_queue, 5, 5, &ctx.slots);

    for button in buttons {
        draw_button(&mut draw_queue, &button, &ctx.mouse)
    }

    draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

    // --- Renderer boilerplate ---
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
    Ok(program_status)
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let full_deck: Vec<PlayingCard> = Suit::iter()
        .flat_map(|suit| Rank::iter().map(move |rank| PlayingCard { suit, rank }))
        .collect();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
        cursor::Hide,
        EnableMouseCapture
    )?;

    let (width, height) = terminal::size()?;

    let mut ctx = Context {
        screen: Screen::new(width, height, (0, 0, 0)),
        mouse: MouseContext {
            x: 0,
            y: 0,
            is_down: false,
        },
        game_time: 0.0,
        slots: Slots {
            spin_count: 0,
            columns: vec![
                Column {
                    cursor: 0.0,
                    cards: full_deck.clone(),
                    spin_duration: 0.0,
                    spin_time_remaining: 0.0,
                    spin_speed: 0.0,
                };
                3
            ],
        },
        fps_counter: FPSCounter::new(0.08),
    };

    let mut fps_limiter = FPSLimiter::new(0.0, 0.001, 0.002);

    'game_loop: loop {
        let dt: f64 = fps_limiter.wait();

        if tick(&mut ctx, &mut stdout)? == ProgramStatus::Exit {
            break 'game_loop;
        }

        ctx.fps_counter.update(dt);
        ctx.game_time += dt;
    }

    terminal::disable_raw_mode()?;
    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        DisableMouseCapture
    )?;

    Ok(())
}
