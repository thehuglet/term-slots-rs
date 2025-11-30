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
mod utils;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{Print, ResetColor, SetStyle},
    terminal::{self},
};
use rand::seq::SliceRandom;
use std::{
    env,
    io::{self, Stdout, Write},
    time::Duration,
};

use crate::{
    button::{Button, draw_button},
    constants::{
        HAND_CARD_X_SPACING, HAND_ORIGIN_X, HAND_ORIGIN_Y, SLOTS_COLUMNS_X_SPACING, SLOTS_ORIGIN_X,
        SLOTS_ORIGIN_Y, TABLE_CARD_X_SPACING, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, TABLE_SLOT_COUNT,
        TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    },
    context::{Context, MouseContext},
    dragged_card::CardDragState,
    fps_counter::{FPSCounter, draw_fps_counter},
    fps_limiter::FPSLimiter,
    hand::{CardInHand, Hand, draw_hand},
    input::{ProgramStatus, drain_input, resolve_input},
    playing_card::{
        BIG_CARD_HEIGHT, BIG_CARD_WIDTH, PlayingCard, Rank, Suit, draw_calls_playing_card_big,
    },
    renderer::{
        Cell, DrawCall, HSL, RGBA, RichText, Screen, ScreenBuffer, blend_source_over,
        build_crossterm_content_style, build_gamma_lut, build_vignette_lut, compose_buffer,
        diff_buffers, draw_rect, fill_screen_background, packed_rgb_to_rgba, rgba_to_packed_rgb,
    },
    slots::{
        Column, Slots, calc_column_spin_duration_sec, draw_slots, slots_stopped, spin_slots_column,
    },
    table::{Table, draw_table},
};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let args: Vec<String> = env::args().collect();
    let mut target_fps = 144.0;

    if let Some(pos) = args.iter().position(|arg| arg == "--fps") {
        if let Some(fps_str) = args.get(pos + 1) {
            if let Ok(fps) = fps_str.parse::<f64>() {
                target_fps = fps;
            }
        }
    }

    let full_deck: Vec<PlayingCard> = Suit::iter()
        .flat_map(|suit| Rank::iter().map(move |rank| PlayingCard { suit, rank }))
        .collect();

    // let (width, height) = terminal::size()?;

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
        cursor::Hide,
        EnableMouseCapture
    )?;

    let mut ctx = Context {
        vignette_lut: build_vignette_lut(
            TERM_SCREEN_WIDTH as usize,
            TERM_SCREEN_HEIGHT as usize,
            1.3,
            2.0,
            1.0,
        ),
        gamma_lut: build_gamma_lut(0.85),
        gamma_correction: false,
        vignette: false,
        screen: Screen::new(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT, (0, 0, 0)),
        mouse: MouseContext {
            x: 0,
            y: 0,
            is_left_down: false,
            card_drag: CardDragState::NotDragging,
        },
        // dragged_card_ctx: DragCardState { card: None },
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
            cards_on_table: vec![],
        },
        hand: Hand {
            hand_size: 10,
            cards_in_hand: vec![],
        },
        fps_counter: FPSCounter::new(0.08),
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

    let mut fps_limiter: FPSLimiter = FPSLimiter::new(target_fps, 0.001, 0.002);

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

fn tick(ctx: &mut Context, dt: f32, stdout: &mut Stdout) -> io::Result<ProgramStatus> {
    // --- Button definitions ---
    let mut buttons: Vec<Button> = vec![];

    buttons.push(Button {
        x: 35,
        y: 3,
        text: format!("${cost} SPIN", cost = 10),
        color: RGBA::from_u8(255, 190, 100, 1.0),
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

    draw_poker_table_background(
        &mut draw_queue,
        0,
        10,
        TERM_SCREEN_WIDTH,
        TERM_SCREEN_HEIGHT - 10,
        ctx.game_time as f32,
    );

    // Hand card background
    for n in 0..ctx.hand.cards_in_hand.len() as u16 {
        draw_rect(
            &mut draw_queue,
            (HAND_ORIGIN_X + n * HAND_CARD_X_SPACING) as i16,
            HAND_ORIGIN_Y as i16,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
            RGBA::from_f32(0.0, 0.08, 0.0, 1.0),
        );
    }

    // Table card background
    for n in 0..TABLE_SLOT_COUNT {
        draw_rect(
            &mut draw_queue,
            (TABLE_ORIGIN_X + n * TABLE_CARD_X_SPACING) as i16,
            TABLE_ORIGIN_Y as i16,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
            RGBA::from_f32(0.0, 0.08, 0.0, 1.0),
        );
    }

    // Slots frame
    let width: u16 = 20 + (ctx.slots.columns.len() as u16 * SLOTS_COLUMNS_X_SPACING);
    for slots_row_index in 0..7 {
        let x: u16 = SLOTS_ORIGIN_X - 5;
        let y: u16 = SLOTS_ORIGIN_Y + slots_row_index - 3;

        let center_row = 3;
        let distance = ((slots_row_index as i16) - center_row).abs() as f32;
        let max_distance = center_row as f32;

        let scale = 1.0 - (distance / max_distance) * 0.3;

        let mut bg_hsl: HSL = RGBA::from_u8(255, 215, 0, 1.0).into();
        bg_hsl.l *= scale;
        bg_hsl.h += -36.0;
        bg_hsl.l *= 0.35;
        bg_hsl.s *= 0.4;

        let fg: RGBA = RGBA::from_u8(0, 0, 0, 1.0);
        let bg: RGBA = bg_hsl.into();

        // Right side shadow
        draw_queue.push(DrawCall {
            x,
            y,
            rich_text: RichText::new(" ".repeat(width.into()))
                .with_fg(fg)
                .with_bg(bg),
        });
    }
    // top & bottom Border
    let half_width: i16 = (width / 2) as i16;
    for y in [1, 9] {
        for x in 0..width as i16 {
            draw_rect(&mut draw_queue, x, y, 1, 1, {
                let mut hsl: HSL = RGBA::from_u8(176, 144, 61, 1.0).into();
                let distance_from_center: i16 = (x - half_width).abs();
                hsl.l *= 0.6 + 0.03 * (half_width - distance_from_center) as f32;
                hsl.s *= 0.8;
                hsl.into()
            });
        }
    }

    // Shadow
    draw_rect(
        &mut draw_queue,
        0,
        (SLOTS_ORIGIN_Y + 5) as i16,
        width,
        1,
        RGBA::from_u8(0, 0, 0, 0.1),
    );

    draw_slots(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y, &ctx.slots);
    draw_table(&mut draw_queue, &ctx, TABLE_ORIGIN_X, TABLE_ORIGIN_Y);
    draw_hand(&mut draw_queue, ctx, HAND_ORIGIN_X, HAND_ORIGIN_Y);

    // Dropshadow on slot columns themselves
    for column_index in 0..ctx.slots.columns.len() as u16 {
        let x: i16 = (SLOTS_ORIGIN_X + column_index * SLOTS_COLUMNS_X_SPACING) as i16;
        let y: i16 = (SLOTS_ORIGIN_Y - 3) as i16;
        let shadow_color: RGBA = RGBA::from_u8(0, 0, 0, 0.1);

        draw_rect(&mut draw_queue, x + 2, y, 1, 7, shadow_color)
    }

    for button in &mut buttons {
        draw_button(&mut draw_queue, &ctx, &button)
    }

    // Card dragging experiment
    if let CardDragState::Dragging { card, source } = &ctx.mouse.card_drag {
        let anchor_x: i16 = ctx.mouse.x as i16 - 1;
        let anchor_y: i16 = ctx.mouse.y as i16 - 2;

        // Card Shadow
        draw_rect(
            &mut draw_queue,
            anchor_x - 1,
            anchor_y + 1,
            BIG_CARD_WIDTH,
            BIG_CARD_HEIGHT,
            RGBA::from_f32(0.0, 0.0, 0.0, 0.13),
        );

        draw_queue.extend(draw_calls_playing_card_big(anchor_x, anchor_y, card));
    }

    draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

    // --- Renderer boilerplate ---
    compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);

    // Post processing step
    if ctx.gamma_correction {
        for cell in ctx.screen.new_buffer.cells.iter_mut() {
            let mut bg = packed_rgb_to_rgba(cell.bg);
            bg.r = ctx.gamma_lut[bg.r as usize];
            bg.g = ctx.gamma_lut[bg.g as usize];
            bg.b = ctx.gamma_lut[bg.b as usize];
            cell.bg = rgba_to_packed_rgb(&bg);

            let mut fg = packed_rgb_to_rgba(cell.fg);
            fg.r = ctx.gamma_lut[fg.r as usize];
            fg.g = ctx.gamma_lut[fg.g as usize];
            fg.b = ctx.gamma_lut[fg.b as usize];
            cell.fg = rgba_to_packed_rgb(&fg);
        }
    }
    if ctx.vignette {
        apply_vignette_lut(&mut ctx.screen.new_buffer, &ctx.vignette_lut);
    }

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
    Ok(ProgramStatus::Running)
}

fn draw_poker_table_background(
    draw_queue: &mut Vec<DrawCall>,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    game_time: f32,
) {
    for x in x..x + w {
        for y in y..y + h {
            let color: RGBA = background_frag(x, y, game_time);
            draw_queue.push(DrawCall {
                x,
                y,
                rich_text: RichText::new(" ")
                    .with_fg(RGBA::from_u8(0, 0, 0, 1.0))
                    .with_bg(color),
            });
        }
    }
}

fn background_frag(x: u16, y: u16, t: f32) -> RGBA {
    let base_color: RGBA = RGBA::from_f32(0.0, 0.2, 0.0, 1.0);

    let frequency: f32 = 1.5;
    let amplitude: f32 = 0.05;

    let cell_x = x / 2;
    let cell_y = y;
    let is_checker: bool = (cell_x + cell_y) % 2 == 0;

    let offset = (cell_x as f32 / 8.0) + (cell_y as f32 / 4.0);
    let phase = ((cell_x as f32 * 12.9898 + cell_y as f32 * 78.233).sin() * 43758.5453).fract()
        * std::f32::consts::PI;
    let t = t * frequency + offset + phase;

    let checker_dim_level = amplitude * t.sin();
    let dim_factor = if is_checker {
        1.0 + checker_dim_level
    } else {
        1.0 - checker_dim_level
    };

    let mut hsl: HSL = base_color.into();
    hsl.l *= dim_factor;
    hsl.l -= 0.02;
    hsl.s *= 0.8;

    let base_swirl = ((cell_x as f32 * 0.3 + cell_y as f32 * 0.5 + t * 0.2).sin()) * 3.0;
    let rand_phase =
        ((cell_x as f32 * 12.34 + cell_y as f32 * 56.78).sin() * 43758.0).fract() * 3.0;
    hsl.h = (hsl.h + base_swirl + rand_phase) % 360.0;

    let bg: RGBA = hsl.into();

    bg
}

fn apply_vignette_lut(buf: &mut ScreenBuffer, lut: &[f32]) {
    let width = buf.width as usize;
    let height = buf.height as usize;
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let alpha = lut[idx];

            let vignette_color = RGBA::from_u8(0, 0, 0, alpha);

            // blend bg
            let old_bg = packed_rgb_to_rgba(buf.cells[idx].bg);
            buf.cells[idx].bg = rgba_to_packed_rgb(&blend_source_over(&old_bg, &vignette_color));

            // optional: blend fg
            let old_fg = packed_rgb_to_rgba(buf.cells[idx].fg);
            buf.cells[idx].fg = rgba_to_packed_rgb(&blend_source_over(&old_fg, &vignette_color));
        }
    }
}
