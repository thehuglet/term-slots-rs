mod button;
mod constants;
mod context;
mod dragged_card;
mod fps_counter;
mod fps_limiter;
mod game_state;
mod hand;
mod input;
mod playing_card;
mod renderer;
mod slots;
mod table;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Print, ResetColor, SetStyle},
    terminal::{self},
};
use rand::seq::SliceRandom;
use std::{
    io::{self, Stdout, Write},
    process::exit,
    time::Duration,
};

use crate::{
    button::{Button, draw_button},
    constants::{HAND_ORIGIN_X, HAND_ORIGIN_Y},
    context::{Context, MouseContext},
    dragged_card::DraggedCardContext,
    fps_counter::{FPSCounter, draw_fps_counter},
    fps_limiter::FPSLimiter,
    hand::{CardInHand, Hand, draw_hand},
    input::{ProgramStatus, resolve_input},
    playing_card::{PlayingCard, Rank, Suit},
    renderer::{
        Cell, DrawCall, HSL, RGBA, RichText, Screen, build_crossterm_content_style, compose_buffer,
        diff_buffers, fill_screen_background,
    },
    slots::{
        Column, Slots, calc_column_spin_duration_sec, draw_slots, slots_stopped, spin_slots_column,
    },
    table::{CardOnTable, Table, draw_table},
};

fn tick(ctx: &mut Context, dt: f32, stdout: &mut Stdout) -> io::Result<ProgramStatus> {
    // --- Button definitions ---
    let mut buttons: Vec<Button> = vec![];

    buttons.push(Button {
        x: 50,
        y: 10,
        text: "SPIN",
        color: RGBA::from_u8(255, 151, 0, 1.0),
        on_click: |ctx: &mut Context| {
            for (column_index, column) in ctx.slots.columns.iter_mut().enumerate() {
                let spin_duration: f32 = calc_column_spin_duration_sec(column_index);
                column.spin_duration = spin_duration;
                column.spin_time_remaining = spin_duration;
            }
        },
        enabled_when: |ctx: &Context| slots_stopped(&ctx.slots),
    });

    // --- Inputs ---
    let program_status: ProgramStatus = if event::poll(Duration::from_millis(0))? {
        resolve_input(ctx, event::read()?, &buttons)
    } else {
        ProgramStatus::Running
    };

    // --- Game logic ---
    for column in &mut ctx.slots.columns {
        const MAX_SPIN_SPEED: f32 = 60.0;
        spin_slots_column(column, dt, MAX_SPIN_SPEED);
    }

    // --- Rendering ---
    fill_screen_background(&mut ctx.screen.new_buffer, (0, 0, 0));
    let mut draw_queue: Vec<DrawCall> = vec![];

    draw_slots(&mut draw_queue, 8, 5, &ctx.slots);
    draw_table(&mut draw_queue, 9, 13, &ctx.table);
    draw_hand(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y, &ctx.hand);

    for button in &mut buttons {
        draw_button(&mut draw_queue, &ctx, &button)
    }

    draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

    // --- Renderer boilerplate ---
    compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);
    let diff: Vec<(u16, u16, &Cell)> = diff_buffers(&ctx.screen.old_buffer, &ctx.screen.new_buffer);

    for (x, y, cell) in diff {
        queue!(
            stdout,
            cursor::MoveTo(x, y),
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
        dragged_card_ctx: DraggedCardContext { card: None },
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
        table: Table {
            cards_on_table: vec![
                CardOnTable {
                    card: PlayingCard {
                        suit: Suit::Spade,
                        rank: Rank::Num10
                    }
                };
                3
            ],
        },
        hand: Hand {
            cursor: 0,
            hand_size: 10,
            cards_in_hand: vec![
                CardInHand {
                    card: PlayingCard {
                        suit: Suit::Heart,
                        rank: Rank::Num2,
                    }
                };
                5
            ],
        },
        fps_counter: FPSCounter::new(0.08),
    };

    for column in &mut ctx.slots.columns {
        column.cards.shuffle(&mut rand::rng());
    }

    let mut fps_limiter = FPSLimiter::new(100.0, 0.001, 0.002);

    'game_loop: loop {
        let dt: f64 = fps_limiter.wait();

        if tick(&mut ctx, dt as f32, &mut stdout)? == ProgramStatus::Exit {
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
