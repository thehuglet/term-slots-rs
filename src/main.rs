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
mod shader;
mod slots;
mod table;
mod utils;

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Print, ResetColor, SetStyle},
    terminal::{self},
};
use rand::seq::SliceRandom;
use std::{
    env,
    io::{self, Stdout, Write},
};

use crate::{
    button::{Button, draw_button},
    constants::{
        BIG_PLAYING_CARD_HEIGHT, BIG_PLAYING_CARD_WIDTH, HAND_CARD_X_SPACING, HAND_ORIGIN_X,
        HAND_ORIGIN_Y, SLOTS_COLUMNS_X_SPACING, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y,
        TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_SLOT_COUNT, TERM_SCREEN_HEIGHT,
        TERM_SCREEN_WIDTH,
    },
    context::Context,
    dragged_card::CardDragState,
    fps_counter::draw_fps_counter,
    fps_limiter::FPSLimiter,
    hand::{CardInHand, draw_hand},
    input::{ProgramStatus, drain_input, resolve_input},
    playing_card::{PlayingCard, Rank, Suit, draw_calls_playing_card_big},
    renderer::{
        Cell, DrawCall, Hsl, Rgba, RichText, build_crossterm_content_style, compose_buffer,
        diff_buffers, draw_rect, fill_screen_background,
    },
    shader::{apply_gamma_lut, apply_vignette, draw_bg_shader},
    slots::{
        calc_column_spin_duration_sec, draw_slots, draw_slots_panel, slots_are_spinning,
        spin_slots_column,
    },
    table::draw_table,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_fps: f32 = args
        .iter()
        .position(|arg| arg == "--fps")
        .and_then(|pos| args.get(pos + 1))
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(144.0);
    let mut fps_limiter: FPSLimiter = FPSLimiter::new(target_fps, 0.001, 0.002);
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
        cursor::Hide,
        EnableMouseCapture
    )?;

    let mut ctx = Context {
        ..Default::default()
    };

    ctx.hand.cards_in_hand = vec![None; ctx.hand.hand_size as usize];
    ctx.table.cards_on_table = vec![None; TABLE_SLOT_COUNT as usize];

    // ! DEBUG !
    // Prefilling card slots
    ctx.hand.cards_in_hand[0] = Some(CardInHand {
        card: PlayingCard {
            suit: Suit::Heart,
            rank: Rank::Num3,
        },
    });
    ctx.hand.cards_in_hand[2] = Some(CardInHand {
        card: PlayingCard {
            suit: Suit::Spade,
            rank: Rank::Num10,
        },
    });
    ctx.hand.cards_in_hand[5] = Some(CardInHand {
        card: PlayingCard {
            suit: Suit::Club,
            rank: Rank::Ace,
        },
    });

    for column in &mut ctx.slots.columns {
        column.cards.shuffle(&mut rand::rng());
    }

    'game_loop: loop {
        let dt: f32 = fps_limiter.wait();

        if tick(&mut ctx, dt as f32, &mut stdout)? == ProgramStatus::Exit {
            break 'game_loop;
        }

        ctx.fps_counter.update(dt);
        ctx.game_time += dt;
    }

    // Exit cleanup
    terminal::disable_raw_mode()?;
    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show,
        DisableMouseCapture
    )?;

    Ok(())
}

fn tick(ctx: &mut Context, dt: f32, stdout: &mut Stdout) -> io::Result<ProgramStatus> {
    // --- Buttons ---
    let mut buttons: Vec<Button> = vec![];

    buttons.push(Button {
        x: 35,
        y: 3,
        text: format!("${cost} SPIN", cost = 10),
        color: Rgba::from_u8(255, 190, 100, 1.0),
        on_click: |ctx: &mut Context| {
            for (column_index, column) in ctx.slots.columns.iter_mut().enumerate() {
                let spin_duration: f32 = calc_column_spin_duration_sec(column_index);
                column.spin_duration = spin_duration;
                column.spin_time_remaining = spin_duration;
            }
        },
        enabled_when: |ctx: &Context| slots_are_spinning(&ctx.slots),
    });

    // --- Inputs ---
    for event in drain_input() {
        if let ProgramStatus::Exit = resolve_input(ctx, event, &buttons) {
            return Ok(ProgramStatus::Exit);
        }
    }

    // --- Game logic ---
    for column in &mut ctx.slots.columns {
        const MAX_SPIN_SPEED: f32 = 60.0;
        spin_slots_column(column, dt, MAX_SPIN_SPEED);
    }

    // --- Rendering ---
    fill_screen_background(&mut ctx.screen.new_buffer, (5, 37, 5));
    let mut draw_queue: Vec<DrawCall> = vec![];

    if ctx.settings.bg_shader_enabled {
        draw_bg_shader(
            &mut draw_queue,
            0,
            10,
            TERM_SCREEN_WIDTH,
            TERM_SCREEN_HEIGHT - 10,
            ctx.game_time,
        );
    }

    draw_slots_panel(&mut draw_queue, 0, 1, 29, 7);

    {
        let color = Rgba::from_u8(0, 0, 0, 0.3);

        // Table card sockets
        for n in 0..TABLE_SLOT_COUNT {
            draw_rect(
                &mut draw_queue,
                (TABLE_ORIGIN_X + n * TABLE_CARD_X_SPACING) as i16,
                TABLE_ORIGIN_Y as i16,
                BIG_PLAYING_CARD_WIDTH,
                BIG_PLAYING_CARD_HEIGHT,
                color,
            );
        }

        // Hand card sockets
        for n in 0..ctx.hand.cards_in_hand.len() as u16 {
            draw_rect(
                &mut draw_queue,
                (HAND_ORIGIN_X + n * HAND_CARD_X_SPACING) as i16,
                HAND_ORIGIN_Y as i16,
                BIG_PLAYING_CARD_WIDTH,
                BIG_PLAYING_CARD_HEIGHT,
                color,
            );
        }
    }

    // Vertical separator
    draw_rect(
        &mut draw_queue,
        30,
        0,
        1,
        TERM_SCREEN_HEIGHT,
        Rgba::from_f32(1.0, 1.0, 1.0, 1.0),
    );

    draw_slots(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y, &ctx.slots);
    draw_table(&mut draw_queue, ctx, TABLE_ORIGIN_X, TABLE_ORIGIN_Y);
    draw_hand(&mut draw_queue, ctx, HAND_ORIGIN_X, HAND_ORIGIN_Y);

    // Dropshadow on slot columns themselves
    for column_index in 0..ctx.slots.columns.len() as u16 {
        let x: i16 = (SLOTS_ORIGIN_X + column_index * SLOTS_COLUMNS_X_SPACING) as i16;
        let y: i16 = (SLOTS_ORIGIN_Y - 3) as i16;
        let shadow_color: Rgba = Rgba::from_u8(0, 0, 0, 0.1);

        draw_rect(&mut draw_queue, x + 2, y, 1, 6, shadow_color)
    }

    for button in &mut buttons {
        draw_button(&mut draw_queue, ctx, button)
    }

    // Card dragging experiment
    if let CardDragState::Dragging { card, .. } = &ctx.mouse.card_drag {
        let anchor_x: i16 = ctx.mouse.x as i16 - 1;
        let anchor_y: i16 = ctx.mouse.y as i16 - 2;

        // Card Shadow
        draw_rect(
            &mut draw_queue,
            anchor_x - 1,
            anchor_y + 1,
            BIG_PLAYING_CARD_WIDTH,
            BIG_PLAYING_CARD_HEIGHT,
            Rgba::from_f32(0.0, 0.0, 0.0, 0.13),
        );

        draw_queue.extend(draw_calls_playing_card_big(anchor_x, anchor_y, card));
    }

    draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

    // --- Renderer boilerplate ---
    compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);

    // Post processing step

    apply_gamma_lut(&mut ctx.screen.new_buffer, &ctx.luts.gamma);

    if ctx.settings.vignette_enabled {
        apply_vignette(&mut ctx.screen.new_buffer, &ctx.luts.vignette);
    }

    let diff: Vec<(u16, u16, &Cell)> = diff_buffers(&ctx.screen.old_buffer, &ctx.screen.new_buffer);

    for (x, y, cell) in diff {
        queue!(
            stdout,
            cursor::MoveTo(x, y),
            SetStyle(build_crossterm_content_style(cell)),
            Print(cell.ch),
            ResetColor,
        )?;
    }

    stdout.flush()?;
    ctx.screen.swap_buffers();
    Ok(ProgramStatus::Running)
}
