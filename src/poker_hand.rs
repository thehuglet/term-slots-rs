use std::cmp::Reverse;

use crate::{
    context::Context,
    playing_card::{PlayingCard, Rank, Suit},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum PokerHand {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

impl PokerHand {
    pub fn repr(&self) -> &'static str {
        match self {
            PokerHand::HighCard => "High Card",
            PokerHand::Pair => "Pair",
            PokerHand::TwoPair => "Two Pair",
            PokerHand::ThreeOfAKind => "Three of a Kind",
            PokerHand::Straight => "Straight",
            PokerHand::Flush => "Flush",
            PokerHand::FullHouse => "Full House",
            PokerHand::FourOfAKind => "Four of a Kind",
            PokerHand::StraightFlush => "Straight Flush",
            PokerHand::RoyalFlush => "Royal Flush",
            PokerHand::FiveOfAKind => "Five of a Kind",
            PokerHand::FlushHouse => "Flush House",
            PokerHand::FlushFive => "Flush Five",
        }
    }
}

// Helper functions that don't modify the original enums
fn suit_to_index(suit: Suit) -> usize {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

fn rank_to_index(rank: Rank) -> usize {
    match rank {
        Rank::Ace => 0,
        Rank::King => 1,
        Rank::Queen => 2,
        Rank::Jack => 3,
        Rank::Num10 => 4,
        Rank::Num9 => 5,
        Rank::Num8 => 6,
        Rank::Num7 => 7,
        Rank::Num6 => 8,
        Rank::Num5 => 9,
        Rank::Num4 => 10,
        Rank::Num3 => 11,
        Rank::Num2 => 12,
    }
}

fn rank_straight_value(rank: Rank) -> i32 {
    match rank {
        Rank::Ace => 14, // Ace high for straight checking
        Rank::King => 13,
        Rank::Queen => 12,
        Rank::Jack => 11,
        Rank::Num10 => 10,
        Rank::Num9 => 9,
        Rank::Num8 => 8,
        Rank::Num7 => 7,
        Rank::Num6 => 6,
        Rank::Num5 => 5,
        Rank::Num4 => 4,
        Rank::Num3 => 3,
        Rank::Num2 => 2,
    }
}

/// A helper wrapper that makes updating the current poker hand easier
pub fn update_current_poker_hand(ctx: &mut Context) {
    let table_cards: Vec<PlayingCard> = ctx
        .table
        .cards_on_table
        .iter()
        .filter_map(|slot| {
            slot.as_ref()
                .map(|card_on_table| card_on_table.card.clone())
        })
        .collect();

    if table_cards.len() == 0 {
        ctx.table.poker_hand = None;
        return;
    }

    let (poker_hand, _): (PokerHand, Vec<PlayingCard>) = eval_poker_hand(&table_cards);
    ctx.table.poker_hand = Some(poker_hand);
}

pub fn eval_poker_hand(cards: &[PlayingCard]) -> (PokerHand, Vec<PlayingCard>) {
    // let n = cards.len();

    // Single pass to build all data structures
    let mut suit_counts = [0u8; 4];
    let mut rank_counts = [0u8; 13];
    let mut cards_by_suit: Vec<Vec<PlayingCard>> = vec![Vec::new(); 4];
    let mut cards_by_rank: Vec<Vec<PlayingCard>> = vec![Vec::new(); 13];

    for card in cards {
        let suit_idx = suit_to_index(card.suit);
        let rank_idx = rank_to_index(card.rank);

        suit_counts[suit_idx] += 1;
        rank_counts[rank_idx] += 1;
        cards_by_suit[suit_idx].push(card.clone());
        cards_by_rank[rank_idx].push(card.clone());
    }

    // Find flush suit and cards (if any)
    let mut flush_suit = None;
    let mut flush_cards = Vec::with_capacity(5);

    for (idx, &count) in suit_counts.iter().enumerate() {
        if count >= 5 {
            flush_suit = Some(idx);
            let mut suit_cards = cards_by_suit[idx].clone();
            // Sort descending by rank
            suit_cards.sort_by_key(|c| Reverse(c.rank));
            flush_cards = suit_cards.into_iter().take(5).collect();
            break;
        }
    }

    let is_flush = flush_suit.is_some();

    // Check for n-of-a-kind patterns
    let mut has_five = false;
    let mut has_four = false;
    let mut has_three = false;
    let mut pair_count = 0;
    let mut best_five_rank_idx = None;
    let mut best_four_rank_idx = None;
    let mut best_three_rank_idx = None;
    let mut pair_rank_indices = Vec::new();

    for (idx, &count) in rank_counts.iter().enumerate() {
        if count == 5 {
            has_five = true;
            best_five_rank_idx = Some(idx);
        } else if count == 4 {
            has_four = true;
            best_four_rank_idx = Some(idx);
        } else if count == 3 {
            has_three = true;
            if best_three_rank_idx.is_none() {
                best_three_rank_idx = Some(idx);
            }
        } else if count == 2 {
            pair_count += 1;
            pair_rank_indices.push(idx);
        }
    }

    // Check for straight
    let straight_info = check_straight(&rank_counts, cards);

    // Check for royal flush
    let is_royal_flush = is_flush && straight_info.is_straight && {
        // Check if lowest rank in straight is 10
        let min_rank_idx = rank_counts.iter().position(|&c| c > 0);
        min_rank_idx == Some(4) // Rank::Num10 = index 4
    };

    // Evaluate hands in exact order from Python version

    // 1. Flush Five
    if is_flush && has_five {
        return (PokerHand::FlushFive, flush_cards);
    }

    // 2. Flush House
    if is_flush && has_three && pair_count >= 1 {
        return (PokerHand::FlushHouse, flush_cards);
    }

    // 3. Five of a Kind
    if has_five {
        if let Some(rank_idx) = best_five_rank_idx {
            let five_cards = cards_by_rank[rank_idx].iter().take(5).cloned().collect();
            return (PokerHand::FiveOfAKind, five_cards);
        }
    }

    // 4. Royal Flush
    if is_royal_flush {
        return (PokerHand::RoyalFlush, flush_cards);
    }

    // 5. Straight Flush
    if is_flush && straight_info.is_straight {
        return (PokerHand::StraightFlush, flush_cards);
    }

    // 6. Four of a Kind
    if has_four {
        if let Some(rank_idx) = best_four_rank_idx {
            let mut four_cards = cards_by_rank[rank_idx]
                .iter()
                .take(4)
                .cloned()
                .collect::<Vec<_>>();
            // Add kicker
            if let Some(kicker) = cards
                .iter()
                .filter(|c| rank_to_index(c.rank) != rank_idx)
                .max_by_key(|c| c.rank)
                .cloned()
            {
                four_cards.push(kicker);
            }
            return (PokerHand::FourOfAKind, four_cards);
        }
    }

    // 7. Full House
    if has_three && pair_count >= 1 {
        let mut full_house_cards = Vec::with_capacity(5);

        // Get three of a kind
        if let Some(three_rank_idx) = best_three_rank_idx {
            full_house_cards.extend(cards_by_rank[three_rank_idx].iter().take(3).cloned());

            // Get best pair (highest rank index is lowest rank value, so we want min index)
            let pair_rank_idx = *pair_rank_indices.iter().min().unwrap(); // Min index = highest rank
            full_house_cards.extend(cards_by_rank[pair_rank_idx].iter().take(2).cloned());

            return (PokerHand::FullHouse, full_house_cards);
        }
    }

    // 8. Flush
    if is_flush {
        return (PokerHand::Flush, flush_cards);
    }

    // 9. Straight
    if straight_info.is_straight {
        return (PokerHand::Straight, straight_info.cards);
    }

    // 10. Three of a Kind
    if has_three {
        if let Some(rank_idx) = best_three_rank_idx {
            let mut three_cards = cards_by_rank[rank_idx]
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>();

            // Add two kickers
            let mut kickers: Vec<PlayingCard> = cards
                .iter()
                .filter(|c| rank_to_index(c.rank) != rank_idx)
                .cloned()
                .collect();

            kickers.sort_by_key(|c| Reverse(c.rank));
            three_cards.extend(kickers.into_iter().take(2));

            return (PokerHand::ThreeOfAKind, three_cards);
        }
    }

    // 11. Two Pair
    if pair_count >= 2 {
        // Sort pair rank indices (lower index = higher rank)
        pair_rank_indices.sort();

        let first_pair_rank_idx = pair_rank_indices[0]; // Lowest index = highest rank
        let second_pair_rank_idx = pair_rank_indices[1];

        let mut two_pair_cards = Vec::with_capacity(5);
        two_pair_cards.extend(cards_by_rank[first_pair_rank_idx].iter().take(2).cloned());
        two_pair_cards.extend(cards_by_rank[second_pair_rank_idx].iter().take(2).cloned());

        // Add kicker
        if let Some(kicker) = cards
            .iter()
            .filter(|c| {
                let idx = rank_to_index(c.rank);
                idx != first_pair_rank_idx && idx != second_pair_rank_idx
            })
            .max_by_key(|c| c.rank)
            .cloned()
        {
            two_pair_cards.push(kicker);
        }

        return (PokerHand::TwoPair, two_pair_cards);
    }

    // 12. Pair
    if pair_count == 1 {
        let pair_rank_idx = pair_rank_indices[0];
        let mut pair_cards = cards_by_rank[pair_rank_idx]
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>();

        // Add three kickers
        let mut kickers: Vec<PlayingCard> = cards
            .iter()
            .filter(|c| rank_to_index(c.rank) != pair_rank_idx)
            .cloned()
            .collect();

        kickers.sort_by_key(|c| Reverse(c.rank));
        pair_cards.extend(kickers.into_iter().take(3));

        return (PokerHand::Pair, pair_cards);
    }

    // 13. High Card
    let highest_card = cards.iter().max_by_key(|c| c.rank).cloned().unwrap();

    (PokerHand::HighCard, vec![highest_card])
}

struct StraightInfo {
    is_straight: bool,
    cards: Vec<PlayingCard>,
}

fn check_straight(rank_counts: &[u8; 13], cards: &[PlayingCard]) -> StraightInfo {
    // Convert to bitmask for fast straight checking
    let mut rank_present = [false; 15]; // Index 2-14 for ranks (2=2, 14=Ace high)

    for card in cards {
        let value = rank_straight_value(card.rank);
        rank_present[value as usize] = true;
    }

    // Check for Ace-low straight (A,2,3,4,5)
    let ace_low_straight = rank_present[14]
        && rank_present[2]
        && rank_present[3]
        && rank_present[4]
        && rank_present[5];

    // Check for regular straights (high to low)
    for high in (5..=14).rev() {
        let mut is_straight = true;
        for i in 0..5 {
            if !rank_present[high - i] {
                is_straight = false;
                break;
            }
        }

        if is_straight {
            // Build straight cards
            let mut straight_cards = Vec::with_capacity(5);
            let straight_ranks = if high == 5 && ace_low_straight {
                // Ace-low straight: A,2,3,4,5
                vec![14, 2, 3, 4, 5]
            } else {
                // Regular straight
                (high - 4..=high).rev().collect::<Vec<_>>()
            };

            // Pick one card per rank
            for &rank_value in &straight_ranks {
                if let Some(card) = cards
                    .iter()
                    .find(|c| rank_straight_value(c.rank) == rank_value as i32)
                {
                    straight_cards.push(card.clone());
                }
            }

            straight_cards.sort_by_key(|c| Reverse(rank_straight_value(c.rank)));
            return StraightInfo {
                is_straight: true,
                cards: straight_cards,
            };
        }
    }

    StraightInfo {
        is_straight: false,
        cards: Vec::new(),
    }
}

impl PokerHand {
    pub fn name(&self) -> &'static str {
        match self {
            PokerHand::FlushFive => "Flush Five",
            PokerHand::FlushHouse => "Flush House",
            PokerHand::FiveOfAKind => "Five of a Kind",
            PokerHand::RoyalFlush => "Royal Flush",
            PokerHand::StraightFlush => "Straight Flush",
            PokerHand::FourOfAKind => "Four of a Kind",
            PokerHand::FullHouse => "Full House",
            PokerHand::Flush => "Flush",
            PokerHand::Straight => "Straight",
            PokerHand::ThreeOfAKind => "Three of a Kind",
            PokerHand::TwoPair => "Two Pair",
            PokerHand::Pair => "Pair",
            PokerHand::HighCard => "High Card",
        }
    }

    pub fn coin_value(&self) -> i32 {
        match self {
            PokerHand::FlushFive => 170,
            PokerHand::FlushHouse => 165,
            PokerHand::FiveOfAKind => 150,
            PokerHand::RoyalFlush => 140,
            PokerHand::StraightFlush => 120,
            PokerHand::FourOfAKind => 100,
            PokerHand::FullHouse => 80,
            PokerHand::Flush => 55,
            PokerHand::Straight => 50,
            PokerHand::ThreeOfAKind => 30,
            PokerHand::TwoPair => 20,
            PokerHand::Pair => 5,
            PokerHand::HighCard => 0,
        }
    }
}
