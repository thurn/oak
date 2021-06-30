// Copyright © 2021-present Derek Thurn

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contains definitions for the core datatypes used in the rest of the game.

use std::fmt;

use strum_macros::EnumIter;

/// Represents the four traditional playing card suits. Note that in Oak the
/// standard suit order is Diamonds < Clubs < Hearts < Spades, different from
/// the ordering used in e.g. Bridge.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter, PartialOrd, Ord)]
pub enum Suit {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

impl Suit {
    /// True if this is a red suit, false if it's black
    fn is_red(&self) -> bool {
        match self {
            Suit::Clubs | Suit::Spades => false,
            Suit::Diamonds | Suit::Hearts => true,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
                Suit::Hearts => "♥",
                Suit::Spades => "♠",
            }
        )
    }
}

/// Represents the standard playing card ranks, with Aces high
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter, PartialOrd, Ord)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "10",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        )
    }
}

/// Represents one of the 52 standard playing cards. Card ordering is by [Suit]
/// first and then by [Rank].
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, PartialOrd, Ord)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }
}

/// Represents one of the four hands in an Oak game.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum Position {
    User,
    Left,
    Dummy,
    Right,
}

impl Position {
    /// Returns the next position in turn sequence after this one
    pub fn next(&self) -> Position {
        match self {
            Position::User => Position::Left,
            Position::Left => Position::Dummy,
            Position::Dummy => Position::Right,
            Position::Right => Position::User,
        }
    }

    /// Returns the partner position of this position
    pub fn partner(&self) -> Position {
        match self {
            Position::User => Position::Dummy,
            Position::Left => Position::Right,
            Position::Dummy => Position::User,
            Position::Right => Position::Left,
        }
    }
}

/// Identifies the current overall state of the game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum GamePhase {
    Bidding,
    Playing,
}

/// Identifier for a [Card] in a given hand
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardId {
    pub position: Position,
    pub index: usize,
}

impl CardId {
    pub fn new(position: Position, index: usize) -> CardId {
        CardId { position, index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_red() {
        assert!(Suit::Hearts.is_red());
        assert!(!Suit::Spades.is_red());
    }

    #[test]
    fn display_suit() {
        assert_eq!(format!("{}", Suit::Hearts), "♥")
    }

    #[test]
    fn display_rank() {
        assert_eq!(format!("{}", Rank::Ten), "10")
    }

    #[test]
    fn position() {
        assert_eq!(Position::Right.next(), Position::User)
    }
}
