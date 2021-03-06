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

//! Types related to bidding

use std::{
    fmt::{self, Display, Formatter},
    ops::RangeInclusive,
};

use strum_macros::EnumIter;

use super::primitives::Rank;
use crate::model::primitives::{Position, Suit};

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Bid {
    Query,
    Suit(Suit),
    Pass,
}

/// A rating of the strength of a hand
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum HandRating {
    Terrible,
    Poor,
    Fair,
    Good,
    Excellent,
    Superb,
}

impl HandRating {
    pub fn new(score: usize) -> Self {
        match score {
            0..=5 => Self::Terrible,
            6..=9 => Self::Poor,
            10..=12 => Self::Fair,
            13..=15 => Self::Good,
            16..=18 => Self::Excellent,
            _ => Self::Superb,
        }
    }

    pub fn approximate_points(&self) -> usize {
        match self {
            Self::Terrible => 5,
            Self::Poor => 8,
            Self::Fair => 10,
            Self::Good => 13,
            Self::Excellent => 16,
            Self::Superb => 19,
        }
    }
}

impl Display for HandRating {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                HandRating::Terrible => "Terrible",
                HandRating::Poor => "Poor",
                HandRating::Fair => "Fair",
                HandRating::Good => "Good",
                HandRating::Excellent => "Excellent",
                HandRating::Superb => "Superb",
            }
        )
    }
}

/// Description of the distribution of a hand. Traditionally a 'balanced hand'
/// is one containing at most one doubleton and no singletons or voids.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum HandBalance {
    Balanced,
    Unbalanced,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum LengthOperator {
    /// Less than or equal to this suit count
    Lte,
    /// Greater than or equal to this suit count
    Gte,
    /// Exactly equal to this suit count
    Equal,
}

impl LengthOperator {
    /// Compares two values, producing [LengthOperator::Lte] or
    /// [LengthOperator::Gte] results, or returning 'bias' if the values
    /// are equal.
    pub fn compare(have: usize, constraint: usize, bias: Self) -> Self {
        match have.cmp(&constraint) {
            std::cmp::Ordering::Less => Self::Lte,
            std::cmp::Ordering::Equal => bias,
            std::cmp::Ordering::Greater => Self::Gte,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum BidResponse {
    /// No response
    Pass,

    /// Hand strength evaluation in points, optionally in the context of a given
    /// trump suit
    HandEvaluation(HandRating, Option<Suit>),

    /// Constraint on the length of a suit, optionally including a hand
    /// evaluation for this trump suit
    SuitLength(Suit, usize, LengthOperator),

    /// Suit distribution
    HandBalance(HandBalance),

    /// Identifies a long suit
    LongestSuit(Suit),

    /// Identifies a weak suit
    WeakestSuit(Suit),

    /// Gives a count of cards with a given [Rank]
    RankCount(Rank, usize),
}

#[derive(Debug, Clone)]
pub struct AuctionTurn {
    pub bid: Bid,
    pub responses: Vec<BidResponse>,
}

impl AuctionTurn {
    pub fn query(response: BidResponse) -> Self {
        Self { bid: Bid::Query, responses: vec![response] }
    }

    pub fn suit(suit: Suit, response: BidResponse) -> Self {
        Self { bid: Bid::Suit(suit), responses: vec![response] }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum Bidder {
    First,
    Second,
}

impl Bidder {
    pub fn opposite(&self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Auction {
    /// Number of tricks the auction winner must win
    pub bid_number: usize,

    /// Position which will act first in bidding
    pub first: Position,
    pub first_bids: Vec<AuctionTurn>,

    /// Position which will act second in bidding
    pub second: Position,
    pub second_bids: Vec<AuctionTurn>,
}

impl Auction {
    pub fn position(&self, bidder: Bidder) -> Position {
        match bidder {
            Bidder::First => self.first,
            Bidder::Second => self.second,
        }
    }

    pub fn bids(&self, bidder: Bidder) -> &Vec<AuctionTurn> {
        match bidder {
            Bidder::First => &self.first_bids,
            Bidder::Second => &self.second_bids,
        }
    }

    pub fn bids_mut(&mut self, bidder: Bidder) -> &mut Vec<AuctionTurn> {
        match bidder {
            Bidder::First => &mut self.first_bids,
            Bidder::Second => &mut self.second_bids,
        }
    }
}
