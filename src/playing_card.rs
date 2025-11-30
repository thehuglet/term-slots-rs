use crate::{
    constants::{DEFAULT_CARD_BG_COLOR, SUIT_COLOR_BLACK, SUIT_COLOR_RED},
    renderer::{DrawCall, RGBA, RichText},
};

pub const BIG_CARD_WIDTH: u16 = 3;
pub const BIG_CARD_HEIGHT: u16 = 3;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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
            Suit::Spade => SUIT_COLOR_BLACK,
            Suit::Heart => SUIT_COLOR_RED,
            Suit::Club => SUIT_COLOR_BLACK,
            Suit::Diamond => SUIT_COLOR_RED,
        }
    }

    pub fn iter() -> std::array::IntoIter<Suit, 4> {
        [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond].into_iter()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

    pub fn iter() -> std::array::IntoIter<Rank, 13> {
        [
            Rank::Ace,
            Rank::King,
            Rank::Queen,
            Rank::Jack,
            Rank::Num10,
            Rank::Num9,
            Rank::Num8,
            Rank::Num7,
            Rank::Num6,
            Rank::Num5,
            Rank::Num4,
            Rank::Num3,
            Rank::Num2,
        ]
        .into_iter()
    }
}

#[derive(Clone)]
pub struct PlayingCard {
    pub suit: Suit,
    pub rank: Rank,
}

pub fn draw_calls_playing_card_small(x: u16, y: u16, card: &PlayingCard) -> DrawCall {
    let suit_repr: &'static str = card.suit.repr();
    let rank_repr: &'static str = card.rank.repr();
    let suit_color: RGBA = card.suit.color();
    let bg_color: RGBA = DEFAULT_CARD_BG_COLOR;

    let text: String = format!("{suit}{rank:>2}", suit = suit_repr, rank = rank_repr);

    DrawCall {
        x: x,
        y: y,
        rich_text: RichText::new(text)
            .with_fg(suit_color)
            .with_bg(bg_color)
            .with_bold(true),
    }
}

pub fn draw_calls_playing_card_big(x: i16, y: i16, card: &PlayingCard) -> Vec<DrawCall> {
    let mut draw_calls: Vec<DrawCall> = vec![];

    let suit_repr: &'static str = card.suit.repr();
    let rank_repr: &'static str = card.rank.repr();
    let suit_color: RGBA = card.suit.color();
    let bg_color: RGBA = DEFAULT_CARD_BG_COLOR;

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
        let line_x: i16 = x.max(0);
        let line_y: i16 = y + row_index as i16;

        // y clipping
        if line_y < 0 {
            continue;
        }

        let mut text_row = pattern_row.to_string();
        text_row = text_row.replace("<<", &format!("{:<2}", rank_repr));
        text_row = text_row.replace(">>", &format!("{:>2}", rank_repr));
        text_row = text_row.replace("S", suit_repr);

        // x clipping
        if x < 0 {
            let chars_to_trim = -x as usize;
            let char_count = text_row.chars().count();

            if chars_to_trim >= char_count {
                continue;
            }

            text_row = text_row.chars().skip(chars_to_trim).collect::<String>();
        }

        let rich_text = RichText::new(text_row)
            .with_fg(suit_color)
            .with_bg(bg_color)
            .with_bold(true);

        draw_calls.push(DrawCall {
            x: line_x as u16,
            y: line_y as u16,
            rich_text,
        });
    }

    draw_calls
}

pub fn get_card_hitbox_rect(
    origin_x: u16,
    origin_y: u16,
    spacing: u16,
    index: usize,
) -> (u16, u16, u16, u16) {
    let x1: u16 = origin_x + index as u16 * spacing;
    let y1: u16 = origin_y;
    let x2: u16 = x1 + BIG_CARD_WIDTH - 1;
    let y2: u16 = y1 + BIG_CARD_HEIGHT - 1;
    (x1, y1, x2, y2)
}
