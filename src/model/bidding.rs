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

use std::ops::RangeInclusive;

use strum_macros::EnumIter;

use super::primitives::Rank;
use crate::model::primitives::{Position, Suit};

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Bid {
    Query,
    Suit(Suit),
    Pass,
}

/// An evaluation of the strength of a hand
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum HandEvaluation {
    Terrible,
    Poor,
    Fair,
    Good,
    Excellent,
    Superb,
}

impl HandEvaluation {
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

    pub fn to_range(self) -> RangeInclusive<usize> {
        match self {
            HandEvaluation::Terrible => 0..=5,
            HandEvaluation::Poor => 6..=9,
            HandEvaluation::Fair => 10..=12,
            HandEvaluation::Good => 13..=15,
            HandEvaluation::Excellent => 16..=18,
            HandEvaluation::Superb => 19..=40,
        }
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

    /// Hand strength evaluation, optionally in the context of a given trump
    /// suit
    HandEvaluation(HandEvaluation, Option<Suit>),

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

#[derive(Debug)]
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

#[derive(Debug)]
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
