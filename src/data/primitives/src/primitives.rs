// Copyright © Oak 2024-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use enum_iterator::Sequence;

/// Represents the four traditional playing card suits.
///
/// Suits are ordered Clubs < Diamonds < Hearts < Spades.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Sequence, PartialOrd, Ord)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Clubs => "♣",
                Suit::Diamonds => "♦",
                Suit::Hearts => "♥",
                Suit::Spades => "♠",
            }
        )
    }
}

/// Represents the standard playing card ranks, with Aces high
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Sequence, PartialOrd, Ord)]
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

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }
}

/// Represents one of the four hands in an Oak game.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Sequence)]
pub enum HandIdentifier {
    /// Dummy partner of human player
    North,
    /// Dummy partner of AI player
    East,
    /// Always the human player in the round
    South,
    /// Always the AI player in the round
    West,
}

impl HandIdentifier {
    /// Returns the next position in turn sequence after this one
    pub fn next(&self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    /// Returns the partner position of this position
    pub fn partner(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub fn player_name(&self) -> PlayerName {
        match self {
            Self::South | Self::North => PlayerName::User,
            Self::East | Self::West => PlayerName::Opponent,
        }
    }
}

/// Identifies one of the two players participating in a round
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Sequence)]
pub enum PlayerName {
    User,
    Opponent,
}
