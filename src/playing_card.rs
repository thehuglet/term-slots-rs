use crate::{
    constants::{COLOR_BLACK, COLOR_RED, DEFAULT_CARD_BG_COLOR},
    renderer::{DrawCall, RGBA, RichText},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Spade,
    Heart,
    Club,
    Diamond,
}

impl Suit {
    pub fn repr(&self) -> &'static str {
        match self {
            Suit::Spade => "♠",
            Suit::Heart => "♥",
            Suit::Club => "♣",
            Suit::Diamond => "♦",
        }
    }

    pub fn color(&self) -> RGBA {
        match self {
            Suit::Spade => COLOR_BLACK,
            Suit::Heart => COLOR_RED,
            Suit::Club => COLOR_BLACK,
            Suit::Diamond => COLOR_RED,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Ace,
    King,
    Queen,
    Jack,
    Num10,
    Num9,
    Num8,
    Num7,
    Num6,
    Num5,
    Num4,
    Num3,
    Num2,
}

impl Rank {
    pub fn repr(&self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::King => "K",
            Rank::Queen => "Q",
            Rank::Jack => "J",
            Rank::Num10 => "10",
            Rank::Num9 => "9",
            Rank::Num8 => "8",
            Rank::Num7 => "7",
            Rank::Num6 => "6",
            Rank::Num5 => "5",
            Rank::Num4 => "4",
            Rank::Num3 => "3",
            Rank::Num2 => "2",
        }
    }
}

pub struct PlayingCard {
    pub suit: Suit,
    pub rank: Rank,
}

pub fn draw_playing_card_big(x: usize, y: usize, card: &PlayingCard) -> Vec<DrawCall> {
    let mut draw_calls = Vec::new();

    let suit_str = card.suit.repr();
    let rank_str = card.rank.repr();
    let suit_color = card.suit.color();
    let bg_color = DEFAULT_CARD_BG_COLOR;

    // Choose pattern based on rank
    let pattern: [&str; 3] = match card.rank {
        #[rustfmt::skip]
        Rank::Ace => [
            "<< ",
            " S ",
            " >>"
        ],
        #[rustfmt::skip]
        Rank::Num2 => [
            "<<S",
            "   ",
            "S>>"
        ],
        #[rustfmt::skip]
        _ => [
            "<<S",
            " S ",
            "S>>"
        ],
    };

    for (row_index, pattern_row) in pattern.iter().enumerate() {
        let mut text_row = pattern_row.to_string();

        text_row = text_row.replace("<<", &format!("{:<2}", rank_str));
        text_row = text_row.replace(">>", &format!("{:>2}", rank_str));
        text_row = text_row.replace("S", suit_str);

        let rich_text = RichText::new(text_row)
            .with_fg(suit_color)
            .with_bg(bg_color)
            .with_bold(true);

        draw_calls.push(DrawCall {
            x,
            y: y + row_index,
            text: rich_text,
        });
    }

    draw_calls
}
