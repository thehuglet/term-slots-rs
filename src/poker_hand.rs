use std::cmp::Reverse;

use crate::{
    card::{Card, Rank, Suit},
    context::Context,
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
    let table_cards: Vec<&Card> = ctx.table.cards_on_table.iter().flatten().collect();

    if table_cards.is_empty() {
        ctx.table.poker_hand = None;
        return;
    }

    let (poker_hand, _): (PokerHand, Vec<Card>) = eval_poker_hand(&table_cards);
    ctx.table.poker_hand = Some(poker_hand);
}

pub fn eval_poker_hand(cards_: &[&Card]) -> (PokerHand, Vec<Card>) {
    let cards: Vec<Card> = cards_.iter().map(|c| **c).collect();
    let mut suit_counts: [u8; 4] = [0u8; 4];
    let mut rank_counts: [u8; 13] = [0u8; 13];
    let mut cards_by_suit: Vec<Vec<Card>> = vec![Vec::new(); 4];
    let mut cards_by_rank: Vec<Vec<Card>> = vec![Vec::new(); 13];

    for card in &cards {
        let suit_index: usize = suit_to_index(card.suit);
        let rank_index: usize = rank_to_index(card.rank);

        suit_counts[suit_index] += 1;
        rank_counts[rank_index] += 1;
        cards_by_suit[suit_index].push(*card);
        cards_by_rank[rank_index].push(*card);
    }

    // Find flush suit and cards (if any)
    let mut flush_suit: Option<usize> = None;
    let mut flush_cards: Vec<Card> = Vec::with_capacity(5);

    for (index, &count) in suit_counts.iter().enumerate() {
        if count >= 5 {
            flush_suit = Some(index);
            let mut suit_cards: Vec<Card> = cards_by_suit[index].clone();
            // Sort descending by rank
            suit_cards.sort_by_key(|card| Reverse(card.rank));
            flush_cards = suit_cards.into_iter().take(5).collect();
            break;
        }
    }

    let is_flush = flush_suit.is_some();

    // Check for n-of-a-kind patterns
    let mut has_five: bool = false;
    let mut has_four: bool = false;
    let mut has_three: bool = false;
    let mut pair_count: i32 = 0;
    let mut best_five_rank_index: Option<usize> = None;
    let mut best_four_rank_index: Option<usize> = None;
    let mut best_three_rank_index: Option<usize> = None;
    let mut pair_rank_indices: Vec<usize> = Vec::new();

    for (rank_index, &rank_count) in rank_counts.iter().enumerate() {
        if rank_count == 5 {
            has_five = true;
            best_five_rank_index = Some(rank_index);
        } else if rank_count == 4 {
            has_four = true;
            best_four_rank_index = Some(rank_index);
        } else if rank_count == 3 {
            has_three = true;
            if best_three_rank_index.is_none() {
                best_three_rank_index = Some(rank_index);
            }
        } else if rank_count == 2 {
            pair_count += 1;
            pair_rank_indices.push(rank_index);
        }
    }

    // Check for straight
    let straight_info: StraightInfo = check_straight(&cards);

    // Check for royal flush
    let is_royal_flush = is_flush && straight_info.is_straight && {
        // Check if lowest rank in straight is 10
        let min_rank_index = rank_counts.iter().position(|&count| count > 0);
        min_rank_index == Some(4) // Rank::Num10 = index 4
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
    if has_five && let Some(rank_index) = best_five_rank_index {
        let five_cards: Vec<Card> = cards_by_rank[rank_index].iter().take(5).cloned().collect();
        return (PokerHand::FiveOfAKind, five_cards);
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
    if has_four && let Some(rank_index) = best_four_rank_index {
        let four_cards: Vec<Card> = cards_by_rank[rank_index]
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        return (PokerHand::FourOfAKind, four_cards);
    }

    // 7. Full House
    if has_three && pair_count >= 1 {
        let mut full_house_cards: Vec<Card> = Vec::with_capacity(5);

        // Get three of a kind
        if let Some(three_rank_index) = best_three_rank_index {
            full_house_cards.extend(cards_by_rank[three_rank_index].iter().take(3).cloned());

            // Get best pair (highest rank index is lowest rank value, so we want min index)
            let pair_rank_index: usize = *pair_rank_indices.iter().min().unwrap(); // Min index = highest rank
            full_house_cards.extend(cards_by_rank[pair_rank_index].iter().take(2).cloned());

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
    if has_three && let Some(rank_index) = best_three_rank_index {
        let three_cards = cards_by_rank[rank_index]
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>();

        return (PokerHand::ThreeOfAKind, three_cards);
    }

    // 11. Two Pair
    if pair_count >= 2 {
        // Sort pair rank indices (lower index = higher rank)
        pair_rank_indices.sort();

        let first_pair_rank_index: usize = pair_rank_indices[0]; // Lowest index = highest rank
        let second_pair_rank_index: usize = pair_rank_indices[1];

        let mut two_pair_cards: Vec<Card> = Vec::with_capacity(5);
        two_pair_cards.extend(cards_by_rank[first_pair_rank_index].iter().take(2).cloned());
        two_pair_cards.extend(
            cards_by_rank[second_pair_rank_index]
                .iter()
                .take(2)
                .cloned(),
        );

        return (PokerHand::TwoPair, two_pair_cards);
    }

    // 12. Pair
    if pair_count == 1 {
        let pair_rank_index: usize = pair_rank_indices[0];
        let pair_cards = cards_by_rank[pair_rank_index]
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>();

        return (PokerHand::Pair, pair_cards);
    }

    // 13. High Card
    let highest_card: Card = cards.iter().max_by_key(|card| card.rank).cloned().unwrap();

    (PokerHand::HighCard, vec![highest_card])
}

struct StraightInfo {
    is_straight: bool,
    cards: Vec<Card>,
}

fn check_straight(cards: &[Card]) -> StraightInfo {
    // Convert to bitmask for fast straight checking
    let mut rank_present: [bool; 15] = [false; 15]; // Index 2-14 for ranks (2=2, 14=Ace high)

    for card in cards {
        let value: i32 = rank_straight_value(card.rank);
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
        let mut is_straight: bool = true;
        for index in 0..5 {
            if !rank_present[high - index] {
                is_straight = false;
                break;
            }
        }

        if is_straight {
            // Build straight cards
            let mut straight_cards: Vec<Card> = Vec::with_capacity(5);
            let straight_ranks: Vec<usize> = if high == 5 && ace_low_straight {
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
                    .find(|card| rank_straight_value(card.rank) == rank_value as i32)
                {
                    straight_cards.push(*card);
                }
            }

            straight_cards.sort_by_key(|card| Reverse(rank_straight_value(card.rank)));
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
