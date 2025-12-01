mod button;
mod constants;
mod context;
mod dragged_card;
mod fps_counter;
mod fps_limiter;
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
        HAND_ORIGIN_X, HAND_ORIGIN_Y, SIDEBAR_BORDER_X, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y,
        TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_SLOT_COUNT, TERM_SCREEN_HEIGHT,
    },
    context::Context,
    dragged_card::{CardDragState, draw_dragged_card},
    fps_counter::draw_fps_counter,
    fps_limiter::FPSLimiter,
    hand::{CardInHand, draw_hand, draw_hand_card_slot},
    input::{ProgramStatus, drain_input, resolve_input},
    playing_card::{PlayingCard, Rank, Suit},
    renderer::{
        Cell, DrawCall, Hsl, Rgba, build_crossterm_content_style, compose_buffer, diff_buffers,
        draw_rect, fill_screen_background,
    },
    shader::{apply_gamma_lut, apply_vignette, draw_bg_shader},
    slots::{
        calc_column_spin_duration_sec, draw_slots, draw_slots_column_shadows, draw_slots_panel,
        slots_are_spinning, spin_slots_column,
    },
    table::{draw_table, draw_table_card_slot},
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
        x: SIDEBAR_BORDER_X + 3,
        y: 5,
        w: 12,
        text: format!("🩨 {cost} SPIN", cost = 10),
        color: Rgba::from_u8(255, 210, 140, 1.0),
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

    // Sidebar
    draw_rect(
        &mut draw_queue,
        SIDEBAR_BORDER_X as i16,
        0,
        17,
        TERM_SCREEN_HEIGHT,
        Rgba::from_u8(37, 16, 16, 1.0),
    );

    if ctx.settings.bg_shader_enabled {
        // Above slots strip
        draw_bg_shader(&mut draw_queue, 0, 0, SIDEBAR_BORDER_X, 1, ctx.game_time);

        // The rest
        let y: u16 = 10;
        draw_bg_shader(
            &mut draw_queue,
            0,
            y,
            SIDEBAR_BORDER_X,
            TERM_SCREEN_HEIGHT - y,
            ctx.game_time,
        );
    }

    draw_slots_panel(&mut draw_queue, 0, 1, 37, 7);
    draw_slots(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y, &ctx.slots);
    draw_slots_column_shadows(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y);

    draw_table_card_slot(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y);
    draw_table(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, ctx);

    draw_hand_card_slot(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y);
    draw_hand(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y, ctx);

    if let CardDragState::Dragging { card, .. } = &ctx.mouse.card_drag {
        draw_dragged_card(&mut draw_queue, ctx.mouse.x, ctx.mouse.y, card);
    }

    draw_sidebar_border(&mut draw_queue, SIDEBAR_BORDER_X);

    for button in &mut buttons {
        draw_button(&mut draw_queue, ctx, button)
    }

    // draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

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

fn draw_sidebar_border(draw_queue: &mut Vec<DrawCall>, x: u16) {
    let half_height: i16 = (TERM_SCREEN_HEIGHT / 2) as i16;

    for y in 0..TERM_SCREEN_HEIGHT as i16 {
        draw_rect(draw_queue, x as i16, y, 1, TERM_SCREEN_HEIGHT, {
            let mut hsl: Hsl = Rgba::from_u8(176, 144, 61, 1.0).into();
            let distance_from_center: i16 = (y - half_height).abs();
            hsl.l *= 0.6 + 0.045 * (half_height - distance_from_center) as f32;
            hsl.s *= 0.8;
            hsl.into()
        });
    }

    // Shadow
    let shadow_width = 2;
    for i in 0..shadow_width {
        let t: f32 = 1.0 - (i as f32 / shadow_width as f32);
        let alpha: f32 = t * 0.1;

        draw_rect(
            draw_queue,
            x as i16 - 1 - i as i16,
            0,
            1,
            TERM_SCREEN_HEIGHT,
            Rgba::from_f32(0.0, 0.0, 0.0, alpha),
        );
    }
}
