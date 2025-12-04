mod button;
mod card;
mod card_ops;
mod card_slot;
mod constants;
mod context;
mod fps_counter;
mod fps_limiter;
mod hand;
mod input;
mod poker_hand;
mod renderer;
mod shader;
mod slot_machine;
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
    cmp, env,
    io::{self, Stdout, Write},
};

use crate::{
    button::{Button, draw_button},
    card::Card,
    card_ops::{CardDragState, draw_dragged_card},
    constants::{
        HAND_ORIGIN_X, HAND_ORIGIN_Y, HAND_SLOT_COUNT, SIDEBAR_BORDER_X, SLOTS_COLUMNS_X_SPACING,
        SLOTS_NEIGHBOR_ROW_COUNT, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y, TABLE_ORIGIN_X, TABLE_ORIGIN_Y,
        TABLE_SLOT_COUNT, TERM_SCREEN_HEIGHT, TERM_SCREEN_WIDTH,
    },
    context::Context,
    fps_counter::draw_fps_counter,
    fps_limiter::FPSLimiter,
    hand::{draw_hand, draw_hand_card_slots},
    input::{ProgramStatus, drain_input, resolve_input},
    poker_hand::{PokerHand, eval_poker_hand, update_current_poker_hand},
    renderer::{
        Cell, DrawCall, Hsl, Rgba, RichText, build_crossterm_content_style, compose_buffer,
        diff_buffers, draw_rect, fill_screen_background,
    },
    shader::{apply_gamma, apply_vignette, draw_bg_shader},
    slot_machine::{
        SlotMachineState, calc_column_spin_duration_sec, draw_slots, draw_slots_column_shadows,
        draw_slots_panel, get_column_card_index, slots_are_spinning,
        slots_center_row_indexes_matching_card, spin_cost, spin_slots_column,
    },
    table::{draw_table, draw_table_card_slots},
    utils::center_text_unicode,
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
        EnableMouseCapture,
        terminal::SetTitle("term-slots"),
        cursor::Hide,
        terminal::SetSize(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT)
    )?;

    let mut ctx = Context {
        ..Default::default()
    };

    ctx.cards_in_hand = vec![None; HAND_SLOT_COUNT as usize];
    ctx.cards_on_table = vec![None; TABLE_SLOT_COUNT as usize];

    // // ! DEBUG !
    // // Prefilling card slots
    // ctx.cards_in_hand_in_hand[0] = Some(CardInHand {
    //     card: PlayingCard {
    //         suit: Suit::Heart,
    //         rank: Rank::Num3,
    //     },
    // });
    // ctx.cards_in_hand_in_hand[1] = Some(CardInHand {
    //     card: PlayingCard {
    //         suit: Suit::Spade,
    //         rank: Rank::Num4,
    //     },
    // });
    // ctx.cards_in_hand_in_hand[2] = Some(CardInHand {
    //     card: PlayingCard {
    //         suit: Suit::Club,
    //         rank: Rank::Num5,
    //     },
    // });
    // ctx.cards_in_hand_in_hand[3] = Some(CardInHand {
    //     card: PlayingCard {
    //         suit: Suit::Club,
    //         rank: Rank::Num6,
    //     },
    // });
    // ctx.cards_in_hand_in_hand[4] = Some(CardInHand {
    //     card: PlayingCard {
    //         suit: Suit::Club,
    //         rank: Rank::Num7,
    //     },
    // });

    for column in ctx.slot_machine.columns.iter_mut() {
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

    // Spin button
    buttons.push(Button {
        x: SIDEBAR_BORDER_X + 3,
        y: 9,
        w: 12,
        h: 1,
        text: format!(
            "${cost} SPIN",
            cost = spin_cost(ctx.slot_machine.spin_count, &ctx.luts.spin_cost)
        ),
        color: Rgba::from_u8(255, 210, 140, 1.0),
        on_click: Box::new(move |ctx| {
            for (column_index, column) in ctx.slot_machine.columns.iter_mut().enumerate() {
                let spin_duration: f32 = calc_column_spin_duration_sec(column_index);
                column.spin_duration = spin_duration;
                column.spin_time_remaining = spin_duration;
            }

            ctx.slot_machine.state = SlotMachineState::Spinning;
            ctx.coins -= spin_cost(ctx.slot_machine.spin_count, &ctx.luts.spin_cost);
            ctx.slot_machine.spin_count += 1;
        }),
        enabled_when: |ctx| {
            let spin_cost: i32 = spin_cost(ctx.slot_machine.spin_count, &ctx.luts.spin_cost);
            matches!(ctx.slot_machine.state, SlotMachineState::Idle) && ctx.coins >= spin_cost
        },
    });

    // Play button
    buttons.push(Button {
        x: SIDEBAR_BORDER_X + 3,
        y: 14,
        w: 12,
        h: 1,
        text: "PLAY".to_string(),
        color: Rgba::from_u8(160, 210, 140, 1.0),
        on_click: Box::new(move |ctx: &mut Context| {
            let table_cards: Vec<&Card> = ctx.cards_on_table.iter().flatten().collect();

            let (poker_hand, scoring_cards): (PokerHand, Vec<Card>) = eval_poker_hand(&table_cards);

            let mut coins_reward_total: u16 = poker_hand.coin_value() as u16;

            // Base score of each card
            for rank in scoring_cards.iter().map(|card| card.rank) {
                coins_reward_total += rank.coin_value()
            }

            ctx.coins += coins_reward_total as i32;
            ctx.score += coins_reward_total as i32;

            // Clear hand
            ctx.cards_on_table = vec![None; TABLE_SLOT_COUNT as usize];
            update_current_poker_hand(ctx);
        }),
        enabled_when: |ctx| {
            let cards_on_table_count = ctx
                .cards_on_table
                .iter()
                .filter(|opt| opt.is_some())
                .count();

            cards_on_table_count > 0
        },
    });

    // Burn button
    buttons.push(Button {
        x: SIDEBAR_BORDER_X + 3,
        y: 16,
        w: 12,
        h: 1,
        text: "BURN".to_string(),
        color: Rgba::from_u8(255, 120, 80, 1.0),
        on_click: Box::new(move |ctx: &mut Context| {
            ctx.cards_on_table = vec![None; TABLE_SLOT_COUNT as usize];
            update_current_poker_hand(ctx);
        }),
        enabled_when: |ctx| {
            let cards_on_table_count = ctx
                .cards_on_table
                .iter()
                .filter(|opt| opt.is_some())
                .count();

            cards_on_table_count > 0
        },
    });

    // Slots post-spin reward buttons
    let cards_in_hand_count: usize = ctx.cards_in_hand.iter().filter(|opt| opt.is_some()).count();

    let cards_on_table_count: usize = ctx
        .cards_on_table
        .iter()
        .filter(|opt| opt.is_some())
        .count();

    let at_least_one_empty_slot_in_hand: bool = cards_in_hand_count < HAND_SLOT_COUNT.into();
    let no_cards_on_table: bool = cards_on_table_count == 0;

    if matches!(ctx.slot_machine.state, SlotMachineState::PostSpin)
        && at_least_one_empty_slot_in_hand
        && no_cards_on_table
    {
        for column_index in 0..ctx.slot_machine.columns.len() {
            let index: usize = column_index;
            buttons.push(Button {
                x: SLOTS_ORIGIN_X + column_index as u16 * SLOTS_COLUMNS_X_SPACING,
                y: SLOTS_ORIGIN_Y - SLOTS_NEIGHBOR_ROW_COUNT as u16,
                w: 3,
                h: 1 + SLOTS_NEIGHBOR_ROW_COUNT as u16 * 2,
                text: "".to_string(),
                color: Rgba::from_u8(0, 0, 0, 0.0),
                on_click: Box::new(move |ctx: &mut Context| {
                    ctx.slot_machine.state = SlotMachineState::Idle;

                    let initial_card = {
                        let column = &ctx.slot_machine.columns[index];
                        let card_index = get_column_card_index(0, column);
                        column.cards[card_index]
                    };

                    let matching_cards: Vec<Card> =
                        slots_center_row_indexes_matching_card(&initial_card, ctx)
                            .into_iter()
                            .map(|col_idx| {
                                let column = &ctx.slot_machine.columns[col_idx];
                                let card_idx = get_column_card_index(0, column);
                                column.cards[card_idx]
                            })
                            .collect();

                    for card in matching_cards {
                        if let Some(empty_slot) =
                            ctx.cards_in_hand.iter_mut().find(|slot| slot.is_none())
                        {
                            *empty_slot = Some(card);
                        } else {
                            // No more empty slots
                            break;
                        }
                    }
                }),
                enabled_when: |_| true,
            });
        }
    }

    // --- Inputs ---
    for event in drain_input() {
        if let ProgramStatus::Exit = resolve_input(ctx, event, &buttons) {
            return Ok(ProgramStatus::Exit);
        }
    }

    // --- Game logic ---
    if matches!(ctx.slot_machine.state, SlotMachineState::Spinning) {
        for column in &mut ctx.slot_machine.columns {
            const MAX_SPIN_SPEED: f32 = 60.0;
            spin_slots_column(column, dt, MAX_SPIN_SPEED);
        }

        if slots_are_spinning(&ctx.slot_machine) {
            ctx.slot_machine.state = SlotMachineState::PostSpin;
        }
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
    draw_slots(
        &mut draw_queue,
        SLOTS_ORIGIN_X,
        SLOTS_ORIGIN_Y,
        &ctx.slot_machine,
        ctx,
    );
    draw_slots_column_shadows(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y);

    draw_table_card_slots(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y);
    draw_table(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, ctx);

    draw_hand_card_slots(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y);
    draw_hand(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y, ctx);

    draw_sidebar_border(&mut draw_queue, SIDEBAR_BORDER_X);

    // Score drawing
    draw_queue.push(DrawCall {
        x: SIDEBAR_BORDER_X + 3,
        y: 3,
        rich_text: RichText::new(format!("{:>12}", ctx.score))
            .with_fg(Rgba::from_u8(190, 230, 255, 1.0))
            .with_bold(true),
    });

    // Used for aligning the currency symbols of all currency displays
    let coin_display_width: u16 = format!("{}", ctx.coins).chars().count() as u16;
    let luck_display_width: u16 = format!("{}", ctx.luck).chars().count() as u16;
    let currency_width: u16 = cmp::max(coin_display_width, luck_display_width);

    // Coin currency drawing
    let coin_formatted = format!("$ {:>width$}", ctx.coins, width = currency_width as usize);
    draw_queue.push(DrawCall {
        x: SIDEBAR_BORDER_X + 3,
        y: 5,
        rich_text: RichText::new(format!("{coin_formatted:>12}"))
            .with_fg(Rgba::from_u8(255, 255, 155, 1.0))
            .with_bold(true),
    });

    // Luck currency drawing
    // let luck_formatted = format!("# {:>width$}", ctx.luck, width = currency_width as usize);
    // draw_queue.push(DrawCall {
    //     x: SIDEBAR_BORDER_X + 3,
    //     y: 6,
    //     rich_text: RichText::new(format!("{:>12}", luck_formatted))
    //         .with_fg(Rgba::from_u8(150, 255, 150, 1.0))
    //         .with_bold(true),
    // });

    // Poker hand preview
    if let Some(poker_hand) = ctx.poker_hand {
        let text_centered: String = if matches!(poker_hand, PokerHand::HighCard) {
            center_text_unicode(poker_hand.repr().to_string(), SIDEBAR_BORDER_X as usize)
        } else {
            center_text_unicode(
                format!(
                    "{poker_hand} (+{bonus_coins})",
                    poker_hand = poker_hand.repr(),
                    bonus_coins = poker_hand.coin_value(),
                ),
                SIDEBAR_BORDER_X as usize,
            )
        };

        draw_queue.push(DrawCall {
            x: 0,
            y: 18,
            rich_text: RichText::new(text_centered).with_bold(true),
        });
    }

    for button in &mut buttons {
        draw_button(&mut draw_queue, ctx, button)
    }

    draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

    if let CardDragState::Dragging { card, .. } = ctx.mouse.card_drag.clone() {
        draw_dragged_card(&mut draw_queue, &card, ctx);
    }

    // --- Renderer boilerplate ---
    compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);

    // Post processing step
    apply_gamma(&mut ctx.screen.new_buffer, &ctx.luts.gamma);
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

    // This doesnt work on linux for some reason
    ctx.resize_update_accumulator += dt;
    if ctx.resize_update_accumulator >= 0.2 {
        ctx.resize_update_accumulator = 0.0;
        queue!(
            stdout,
            terminal::SetSize(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT)
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
            let distance_from_center: i16 = (y - half_height + 1).abs();
            hsl.l *= 0.6 + 0.045 * (half_height - distance_from_center) as f32;
            hsl.s *= 0.8;
            hsl.into()
        });
    }

    // Shadow
    let shadow_width: i32 = 2;
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
