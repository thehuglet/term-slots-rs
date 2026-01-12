// mod button;
// mod card;
// mod card_ops;
// mod card_slot;
// mod constants;
// mod context;
// mod fps_counter;
// mod fps_limiter;
// mod hand;
// mod input;
// mod poker_hand;
// mod renderer;
// mod shader;
// mod slot_machine;
// mod table;
// mod utils;
mod engine;
mod renderer_old;

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::engine::{
    Engine, Pos, Size,
    color::Color,
    draw::{draw_fps_counter, draw_rect, draw_text, fill_screen},
    end_frame, exit_cleanup, init,
    input::poll_input,
    rich_text::{Attributes, RichText},
    start_frame,
};

// use crossterm::{
//     cursor,
//     event::{DisableMouseCapture, EnableMouseCapture},
//     execute, queue,
//     style::{Print, ResetColor, SetStyle},
//     terminal::{self},
// };
// use rand::seq::SliceRandom;
// use std::{
//     cmp, env,
//     io::{self, Stdout, Write},
// };

// use crate::{
//     button::{Button, draw_button},
//     card::Card,
//     card_ops::{CardDragState, draw_dragged_card},
//     constants::SIDEBAR_BORDER_X,
//     context::{Context, ImpulseId},
//     fps_counter::{draw_fps_counter, update_fps_counter},
//     fps_limiter::{FPSLimiter, wait_for_next_frame},
//     hand::{HAND_ORIGIN_X, HAND_ORIGIN_Y, draw_hand, draw_hand_card_slots},
//     input::{ProgramStatus, drain_input, resolve_input},
//     poker_hand::{PokerHand, eval_poker_hand, update_current_poker_hand},
//     renderer::{
//         Cell, DrawCall, Hsl, Rgba, RichText, build_crossterm_content_style, compose_buffer,
//         diff_buffers, draw_rect, draw_text, fill_screen_background,
//     },
//     shader::{apply_gamma, apply_vignette, draw_bg_shader},
//     slot_machine::{
//         SLOTS_COLUMNS_X_SPACING, SLOTS_NEIGHBOR_ROW_COUNT, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y,
//         SlotMachineState, calc_column_spin_duration_sec, draw_slots, draw_slots_column_shadows,
//         draw_slots_panel, get_column_card_index, slots_are_spinning,
//         slots_center_row_indexes_matching_card, spin_cost, spin_slots_column,
//     },
//     table::{TABLE_ORIGIN_X, TABLE_ORIGIN_Y, draw_table, draw_table_card_slots},
//     utils::center_text_unicode,
// };

pub const TERM_COLS: u16 = 30;
pub const TERM_ROWS: u16 = 20;

fn main() -> io::Result<()> {
    let mut engine: Engine = Engine::new(TERM_COLS, TERM_ROWS)
        .title("term-slots-rs")
        .limit_fps(0);

    init(&mut engine)?;

    'game_loop: loop {
        start_frame(&mut engine);

        fill_screen(&mut engine, Color::new(80, 80, 80, 255));
        draw_fps_counter(&mut engine, Pos::new(0, 0));

        for event in poll_input() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                println!("Quit!");
                break 'game_loop;
            }
        }

        // draw_text(&mut engine, Pos::new(-3, 3), "a-b-c-d-e-f-g-h-i");

        // Regular bg blending test
        // draw_text(
        //     &mut engine,
        //     Pos::new(3, 5),
        //     RichText::new(" ")
        //         .fg(Color::new(0, 0, 0, 255))
        //         .bg(Color::BLACK),
        // );

        draw_rect(
            &mut engine,
            Pos::square(1, 4),
            Size::square(2, 2),
            Color::BLACK,
        );
        draw_text(
            &mut engine,
            Pos::square(1, 4),
            RichText::new("ab")
                .fg(Color::WHITE)
                .attributes(Attributes::ITALIC),
        );

        draw_text(
            &mut engine,
            Pos::square(2, 4),
            RichText::new("@@")
                .fg(Color::RED)
                .attributes(Attributes::BOLD),
        );

        draw_text(
            &mut engine,
            Pos::square(2, 5),
            RichText::new("cd")
                .fg(Color::WHITE)
                .attributes(Attributes::BOLD),
        );
        draw_rect(
            &mut engine,
            Pos::square(2, 3),
            Size::square(2, 2),
            Color::new(255, 255, 255, 127),
        );
        draw_text(
            &mut engine,
            Pos::square(2, 3),
            RichText::new("ab")
                .fg(Color::BLACK)
                .attributes(Attributes::ITALIC),
        );
        draw_text(
            &mut engine,
            Pos::square(3, 4),
            RichText::new("cd")
                .fg(Color::BLACK)
                .attributes(Attributes::BOLD),
        );

        draw_rect(
            &mut engine,
            Pos::square(5, 4),
            Size::square(2, 2),
            Color::new(255, 255, 255, 127),
        );

        draw_text(
            &mut engine,
            Pos::square(5, 5),
            RichText::new("AB").fg(Color::WHITE),
        );

        draw_rect(
            &mut engine,
            Pos::square(6, 3),
            Size::square(2, 2),
            Color::new(255, 0, 0, 127),
        );
        draw_text(
            &mut engine,
            Pos::square(7, 3),
            RichText::new("CD").fg(Color::RED),
        );

        // Box shadow over bg and fg test
        draw_rect(
            &mut engine,
            Pos::square(9, 4),
            Size::square(2, 2),
            Color::WHITE,
        );
        draw_text(
            &mut engine,
            Pos::square(9, 4),
            RichText::new("ABCD").fg(Color::GREEN),
        );
        draw_rect(
            &mut engine,
            Pos::square(10, 3),
            Size::square(2, 2),
            Color::new(0, 0, 0, 150),
        );

        end_frame(&mut engine)?;
    }

    exit_cleanup(&mut engine)?;
    Ok(())
}

// fn main() -> io::Result<()> {
//     let args: Vec<String> = env::args().collect();
//     let target_fps: f32 = args
//         .iter()
//         .position(|arg| arg == "--fps")
//         .and_then(|pos| args.get(pos + 1))
//         .and_then(|s| s.parse::<f32>().ok())
//         .unwrap_or(144.0);
//     let mut fps_limiter: FPSLimiter = FPSLimiter::new(target_fps, 0.001, 0.002);
//     let mut stdout = io::stdout();

//     terminal::enable_raw_mode()?;
//     execute!(
//         stdout,
//         terminal::EnterAlternateScreen,
//         terminal::DisableLineWrap,
//         EnableMouseCapture,
//         terminal::SetTitle("term-slots"),
//         cursor::Hide,
//         terminal::SetSize(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT)
//     )?;

//     let mut ctx = Context {
//         ..Default::default()
//     };

//     // ! DEBUG !
//     // Prefilling card slots
//     // for index in 0..7 {
//     //     ctx.hand_card_slots[index].card = Some(Card {
//     //         suit: Suit::Heart,
//     //         rank: Rank::Num3,
//     //     });
//     // }
//     // ctx.slot_machine.state = SlotMachineState::PostSpin;

//     for column in ctx.slot_machine.columns.iter_mut() {
//         column.cards.shuffle(&mut rand::rng());
//     }

//     'game_loop: loop {
//         let dt: f32 = wait_for_next_frame(&mut fps_limiter);

//         if tick(&mut ctx, dt as f32, &mut stdout)? == ProgramStatus::Exit {
//             break 'game_loop;
//         }

//         update_fps_counter(&mut ctx.fps_counter, dt);
//         ctx.game_time += dt;
//     }

//     // Exit cleanup
//     terminal::disable_raw_mode()?;
//     execute!(
//         stdout,
//         terminal::LeaveAlternateScreen,
//         cursor::Show,
//         DisableMouseCapture
//     )?;

//     Ok(())
// }

// fn tick(ctx: &mut Context, dt: f32, stdout: &mut Stdout) -> io::Result<ProgramStatus> {
//     // --- Buttons ---
//     let mut buttons: Vec<Button> = vec![];

//     // Spin button
//     buttons.push(Button {
//         x: SIDEBAR_BORDER_X + 3,
//         y: 9,
//         w: 12,
//         h: 1,
//         text: format!(
//             "${cost} SPIN",
//             cost = spin_cost(ctx.slot_machine.spin_count),
//         ),
//         color: Rgba::from_u8(255, 210, 140, 1.0),
//         on_click: Box::new(move |ctx| {
//             for (column_index, column) in ctx.slot_machine.columns.iter_mut().enumerate() {
//                 let spin_duration: f32 = calc_column_spin_duration_sec(column_index);
//                 column.spin_duration = spin_duration;
//                 column.spin_time_remaining = spin_duration;
//             }

//             ctx.slot_machine.state = SlotMachineState::Spinning;
//             ctx.coins -= spin_cost(ctx.slot_machine.spin_count);
//             ctx.slot_machine.spin_count += 1;
//         }),
//         enabled_when: |ctx| {
//             let spin_cost: i32 = spin_cost(ctx.slot_machine.spin_count);
//             matches!(ctx.slot_machine.state, SlotMachineState::Idle) && ctx.coins >= spin_cost
//         },
//         allow_rmb: false,
//     });

//     // Play button
//     buttons.push(Button {
//         x: SIDEBAR_BORDER_X + 3,
//         y: 14,
//         w: 12,
//         h: 1,
//         text: "PLAY".to_string(),
//         color: Rgba::from_u8(160, 210, 140, 1.0),
//         on_click: Box::new(move |ctx: &mut Context| {
//             let cards: Vec<&Card> = ctx
//                 .table_card_slots
//                 .iter()
//                 .filter_map(|slot| slot.card.as_ref())
//                 .collect();

//             let (poker_hand, scoring_cards): (PokerHand, Vec<Card>) = eval_poker_hand(&cards);

//             let mut coins_reward_total: u16 = poker_hand.coin_value() as u16;

//             // Base score of each card
//             for rank in scoring_cards.iter().map(|card| card.rank) {
//                 coins_reward_total += rank.coin_value()
//             }

//             ctx.coins += coins_reward_total as i32;
//             ctx.score += coins_reward_total as i32;

//             // Clear hand
//             ctx.table_card_slots.iter_mut().for_each(|slot| {
//                 slot.card = None;
//             });
//             update_current_poker_hand(ctx);
//         }),
//         enabled_when: |ctx| {
//             let any_cards_on_table: bool =
//                 ctx.table_card_slots.iter().any(|slot| slot.card.is_some());
//             any_cards_on_table
//         },
//         allow_rmb: false,
//     });

//     // Burn button
//     buttons.push(Button {
//         x: SIDEBAR_BORDER_X + 3,
//         y: 16,
//         w: 12,
//         h: 1,
//         text: "BURN".to_string(),
//         color: Rgba::from_u8(255, 120, 80, 1.0),
//         on_click: Box::new(move |ctx: &mut Context| {
//             ctx.table_card_slots.iter_mut().for_each(|slot| {
//                 slot.card = None;
//             });
//             update_current_poker_hand(ctx);
//         }),
//         enabled_when: |ctx| {
//             let any_cards_on_table: bool =
//                 ctx.table_card_slots.iter().any(|slot| slot.card.is_some());
//             any_cards_on_table
//         },
//         allow_rmb: false,
//     });

//     // Slots post-spin reward buttons
//     // let cards_in_hand_count: usize = ctx
//     //     .hand_card_slots
//     //     .iter()
//     //     .filter(|slot| slot.card.is_some())
//     //     .count();

//     // let cards_on_table_count: usize = ctx
//     //     .table_card_slots
//     //     .iter()
//     //     .filter(|slot| slot.card.is_some())
//     //     .count();

//     // let at_least_one_empty_slot_in_hand: bool = cards_in_hand_count < HAND_SLOT_COUNT.into();
//     // let no_cards_on_table: bool = cards_on_table_count == 0;

//     if matches!(ctx.slot_machine.state, SlotMachineState::PostSpin) {
//         for column_index in 0..ctx.slot_machine.columns.len() {
//             let index: usize = column_index;
//             buttons.push(Button {
//                 x: SLOTS_ORIGIN_X + column_index as u16 * SLOTS_COLUMNS_X_SPACING,
//                 y: SLOTS_ORIGIN_Y - SLOTS_NEIGHBOR_ROW_COUNT as u16,
//                 w: 3,
//                 h: 1 + SLOTS_NEIGHBOR_ROW_COUNT as u16 * 2,
//                 text: "".to_string(),
//                 color: Rgba::from_u8(0, 0, 0, 0.0),
//                 on_click: Box::new(move |ctx: &mut Context| {
//                     let empty_hand_slot_count = ctx
//                         .hand_card_slots
//                         .iter()
//                         .filter(|slot| slot.card.is_none())
//                         .count();

//                     if empty_hand_slot_count == 0 {
//                         ctx.impulse_timestamps
//                             .insert(ImpulseId::NoSpaceInHandHint, ctx.game_time);
//                         return;
//                     }

//                     ctx.slot_machine.state = SlotMachineState::Idle;

//                     let clicked_column = &ctx.slot_machine.columns[index];
//                     let clicked_card_index = get_column_card_index(0, clicked_column);
//                     let clicked_card = clicked_column.cards[clicked_card_index];

//                     // Get all matching indexes
//                     let all_matching_indexes =
//                         slots_center_row_indexes_matching_card(&clicked_card, ctx);

//                     // Calculate how many cards we can actually take
//                     let max_cards_to_take = empty_hand_slot_count.min(all_matching_indexes.len());

//                     // Take the clicked card and then other matching cards
//                     let mut cards_to_take = Vec::new();

//                     // Always take the clicked card if we can take at least 1
//                     cards_to_take.push(clicked_card);

//                     // Then take up to (max_cards_to_take - 1) other matching cards
//                     let other_matching_cards: Vec<Card> = all_matching_indexes
//                         .iter()
//                         .filter(|&&col_idx| col_idx != index)
//                         .take(max_cards_to_take.saturating_sub(1))
//                         .map(|&col_idx| {
//                             let column = &ctx.slot_machine.columns[col_idx];
//                             let card_idx = get_column_card_index(0, column);
//                             column.cards[card_idx]
//                         })
//                         .collect();

//                     cards_to_take.extend(other_matching_cards);

//                     // Put cards in hand
//                     for card in cards_to_take {
//                         if let Some(empty_slot) = ctx
//                             .hand_card_slots
//                             .iter_mut()
//                             .find(|slot| slot.card.is_none())
//                         {
//                             empty_slot.card = Some(card);
//                         }
//                     }
//                 }),
//                 enabled_when: |_| true,
//                 allow_rmb: true,
//             });
//         }
//     }

// // --- Inputs ---
// for event in drain_input() {
//     if let ProgramStatus::Exit = resolve_input(ctx, event, &buttons) {
//         return Ok(ProgramStatus::Exit);
//     }
// }

//     // --- Game logic ---
//     if matches!(ctx.slot_machine.state, SlotMachineState::Spinning) {
//         for column in &mut ctx.slot_machine.columns {
//             const MAX_SPIN_SPEED: f32 = 60.0;
//             spin_slots_column(column, dt, MAX_SPIN_SPEED);
//         }

//         if slots_are_spinning(&ctx.slot_machine) {
//             ctx.slot_machine.state = SlotMachineState::PostSpin;
//         }
//     }

//     // --- Rendering ---
//     fill_screen_background(&mut ctx.screen.new_buffer, (5, 37, 5));
//     let mut draw_queue: Vec<DrawCall> = vec![];

//     // Sidebar
//     draw_rect(
//         &mut draw_queue,
//         SIDEBAR_BORDER_X as i16,
//         0,
//         17,
//         TERM_SCREEN_HEIGHT,
//         Rgba::from_u8(37, 16, 16, 1.0),
//     );

//     if ctx.settings.bg_shader_enabled {
//         // Above slots strip
//         draw_bg_shader(&mut draw_queue, 0, 0, SIDEBAR_BORDER_X, 1, ctx.game_time);

//         // Main play area
//         let y: u16 = 10;
//         draw_bg_shader(
//             &mut draw_queue,
//             0,
//             y,
//             SIDEBAR_BORDER_X,
//             TERM_SCREEN_HEIGHT - y,
//             ctx.game_time,
//         );
//     }

//     draw_slots_panel(&mut draw_queue, 0, 1, 37, 7);
//     draw_slots(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y, ctx);
//     draw_slots_column_shadows(&mut draw_queue, SLOTS_ORIGIN_X, SLOTS_ORIGIN_Y);

//     draw_table_card_slots(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y);
//     draw_table(&mut draw_queue, TABLE_ORIGIN_X, TABLE_ORIGIN_Y, ctx);

//     draw_hand_card_slots(&mut draw_queue, HAND_ORIGIN_X, HAND_ORIGIN_Y);
//     draw_hand(&mut draw_queue, ctx);

//     draw_sidebar_border(&mut draw_queue, SIDEBAR_BORDER_X);

//     // Score drawing
//     draw_text(
//         &mut draw_queue,
//         SIDEBAR_BORDER_X + 3,
//         3,
//         RichText::new(format!("{:>12}", ctx.score))
//             .with_fg(Rgba::from_u8(190, 230, 255, 1.0))
//             .with_bold(true),
//     );

//     // Used for aligning the currency symbols of all currency displays
//     let coin_display_width: u16 = format!("{}", ctx.coins).chars().count() as u16;
//     let luck_display_width: u16 = format!("{}", ctx.luck).chars().count() as u16;
//     let currency_width: u16 = cmp::max(coin_display_width, luck_display_width);

//     // Coin currency drawing
//     let coin_formatted: String =
//         format!("$ {:>width$}", ctx.coins, width = currency_width as usize);
//     let coin_amount_rich_text = RichText::new(format!("{coin_formatted:>12}"))
//         .with_fg(Rgba::from_u8(255, 255, 155, 1.0))
//         .with_bold(true);
//     draw_text(
//         &mut draw_queue,
//         SIDEBAR_BORDER_X + 3,
//         5,
//         coin_amount_rich_text,
//     );

//     // Luck currency drawing
//     let luck_formatted: String = format!("# {:>width$}", ctx.luck, width = currency_width as usize);
//     let luck_amount_rich_text = RichText::new(format!("{:>12}", luck_formatted))
//         .with_fg(Rgba::from_u8(150, 255, 150, 1.0))
//         .with_bold(true);
//     draw_text(
//         &mut draw_queue,
//         SIDEBAR_BORDER_X + 3,
//         6,
//         luck_amount_rich_text,
//     );

//     // Poker hand preview
//     if let Some(poker_hand) = ctx.poker_hand {
//         let text_centered: String = if matches!(poker_hand, PokerHand::HighCard) {
//             center_text_unicode(poker_hand.repr().to_string(), SIDEBAR_BORDER_X as usize)
//         } else {
//             center_text_unicode(
//                 format!(
//                     "{poker_hand} (+{bonus_coins})",
//                     poker_hand = poker_hand.repr(),
//                     bonus_coins = poker_hand.coin_value(),
//                 ),
//                 SIDEBAR_BORDER_X as usize,
//             )
//         };

//         draw_text(
//             &mut draw_queue,
//             0,
//             18,
//             RichText::new(text_centered).with_bold(true),
//         );
//     }

//     for button in &mut buttons {
//         draw_button(&mut draw_queue, ctx, button)
//     }

//     draw_fps_counter(&mut draw_queue, 0, 0, &ctx.fps_counter);

//     if let CardDragState::Dragging { card, .. } = ctx.mouse.card_drag.clone() {
//         draw_dragged_card(&mut draw_queue, &card, ctx);
//     }

//     // --- Renderer boilerplate ---
//     compose_buffer(&mut ctx.screen.new_buffer, &draw_queue);

//     // Post processing step
//     apply_gamma(&mut ctx.screen.new_buffer, &ctx.luts.gamma);
//     if ctx.settings.vignette_enabled {
//         apply_vignette(&mut ctx.screen.new_buffer, &ctx.luts.vignette);
//     }

//     let diff: Vec<(u16, u16, &Cell)> = diff_buffers(&ctx.screen.old_buffer, &ctx.screen.new_buffer);

// for (x, y, cell) in diff {
//     queue!(
//         stdout,
//         cursor::MoveTo(x, y),
//         SetStyle(build_crossterm_content_style(cell)),
//         Print(cell.ch),
//         ResetColor,
//     )?;
// }

//     // This doesnt work on linux for some reason
//     ctx.resize_update_accumulator += dt;
//     if ctx.resize_update_accumulator >= 0.2 {
//         ctx.resize_update_accumulator = 0.0;
//         queue!(
//             stdout,
//             terminal::SetSize(TERM_SCREEN_WIDTH, TERM_SCREEN_HEIGHT)
//         )?;
//     }

//     stdout.flush()?;
//     ctx.screen.swap_buffers();
//     Ok(ProgramStatus::Running)
// }

// fn draw_sidebar_border(draw_queue: &mut Vec<DrawCall>, x: u16) {
//     let half_height: i16 = (TERM_SCREEN_HEIGHT / 2) as i16;

//     for y in 0..TERM_SCREEN_HEIGHT as i16 {
//         draw_rect(draw_queue, x as i16, y, 1, TERM_SCREEN_HEIGHT, {
//             let mut hsl: Hsl = Rgba::from_u8(176, 144, 61, 1.0).into();
//             let distance_from_center: i16 = (y - half_height + 1).abs();
//             hsl.l *= 0.6 + 0.045 * (half_height - distance_from_center) as f32;
//             hsl.s *= 0.8;
//             hsl.into()
//         });
//     }

//     // Shadow
//     let shadow_width: i32 = 2;
//     for i in 0..shadow_width {
//         let t: f32 = 1.0 - (i as f32 / shadow_width as f32);
//         let alpha: f32 = t * 0.1;

//         draw_rect(
//             draw_queue,
//             x as i16 - 1 - i as i16,
//             0,
//             1,
//             TERM_SCREEN_HEIGHT,
//             Rgba::from_f32(0.0, 0.0, 0.0, alpha),
//         );
//     }
// }
