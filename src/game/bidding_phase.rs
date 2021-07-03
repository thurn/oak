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

//! Functions for implementing the bidding phase of a game

use std::cmp::Ordering;

use anyhow::{anyhow, Result};
use strum::IntoEnumIterator;

use crate::{
    agents::agent::Agent,
    model::{
        bidding::{Auction, AuctionTurn, Bid, BidResponse, Bidder, HandEvaluation, LengthOperator},
        game::GameData,
        primitives::{Card, Position, Rank, Suit, SuitCounts},
        state::State,
    },
};

/// Returns true if this [Bidder] has passed in the auction
pub fn has_passed(auction: &Auction, bidder: Bidder) -> bool {
    auction.bids(bidder).iter().any(|b| b.bid == Bid::Pass)
}

/// Returns true if both [Bidder]s have passed in the auction
pub fn is_completed(auction: &Auction) -> bool {
    has_passed(auction, Bidder::First) && has_passed(auction, Bidder::Second)
}

/// Returns the [Bidder] which should make the next bid, or None if bidding has
/// been completed
pub fn next_to_bid(auction: &Auction) -> Option<Bidder> {
    match (has_passed(auction, Bidder::First), has_passed(auction, Bidder::Second)) {
        (true, true) => None,
        (true, false) => Some(Bidder::Second),
        (false, true) => Some(Bidder::First),
        (false, false) => {
            if auction.bids(Bidder::First).len() > auction.bids(Bidder::Second).len() {
                Some(Bidder::Second)
            } else {
                Some(Bidder::First)
            }
        }
    }
}

/// Returns the number of bids of a given [Bid] type this [Bidder] has already
/// placed
pub fn bids_of_type(auction: &Auction, bidder: Bidder, bid: Bid) -> usize {
    auction.bids(bidder).iter().filter(|turn| turn.bid == bid).count()
}

/// Evaluates the strength of a hand, optionally adding short suit points for a
/// known trump suit.
#[allow(clippy::option_if_let_else)]
pub fn hand_evaluation(hand: &[Card], trump_suit: Option<Suit>) -> (HandEvaluation, SuitCounts) {
    let mut score = 0;
    let mut counts = SuitCounts::default();

    for card in hand {
        score += match card.rank {
            Rank::Ace => 4,
            Rank::King => 3,
            Rank::Queen => 2,
            Rank::Jack => 1,
            _ => 0,
        };
        counts.increment(card.suit)
    }

    if let Some(trump) = trump_suit {
        // Add short suit points, void = 5, singleton = 3, doubleton = 1
        let short_suit_points = Suit::iter()
            .map(|suit| match counts.count(suit) {
                _ if suit == trump => 0,
                0 => 5,
                1 => 3,
                2 => 1,
                _ => 0,
            })
            .sum::<usize>();
        (HandEvaluation::new(score + short_suit_points), counts)
    } else {
        (HandEvaluation::new(score), counts)
    }
}

/// Returns the [BidResponse] for a [Bid::Query] bid
pub fn query_bid_response(game: &GameData, bidder: Bidder) -> BidResponse {
    BidResponse::Pass
}

/// Returns the result given in the most recent [BidResponse::SuitLength] for
/// this [Bidder], if any
pub fn previous_suit_response(
    game: &GameData,
    bidder: Bidder,
    suit: Suit,
) -> Option<(usize, LengthOperator)> {
    game.auction
        .bids(bidder)
        .iter()
        .rev()
        .filter_map(|AuctionTurn { response, .. }| match response {
            BidResponse::SuitLength(s, len, op, _) if *s == suit => Some((*len, *op)),
            _ => None,
        })
        .next()
}

/// Construct a new [BidResponse::SuitLength] by comparing the length in hand
/// with a target, returning 'bias' if they are equal, along with an optional
/// [HandEvaluation].
pub fn suit_length(
    suit: Suit,
    have: usize,
    constraint: usize,
    bias: LengthOperator,
    evaluation: Option<HandEvaluation>,
) -> BidResponse {
    let op = match have.cmp(&constraint) {
        Ordering::Less => LengthOperator::Lower,
        Ordering::Equal => bias,
        Ordering::Greater => LengthOperator::Higher,
    };

    BidResponse::SuitLength(suit, constraint, op, evaluation)
}

/// Returns the [BidResponse] for a [Bid::Suit] bid
pub fn suit_bid_response(game: &GameData, bidder: Bidder, suit: Suit) -> BidResponse {
    let (eval, suit_count) =
        hand_evaluation(game.hand(game.auction.position(bidder).partner()), Some(suit));
    let count = suit_count.count(suit);

    if let Some((prev, op)) = previous_suit_response(game, bidder, suit) {
        // If we've previously responded for this suit, we increment or decrement our
        // response by 1. If this would reveal the exact count, we provide an EqualTo
        // response.
        match op {
            _ if prev == count => suit_length(suit, count, prev, LengthOperator::Equal, Some(eval)),
            LengthOperator::Lower => suit_length(suit, count, prev - 1, op, Some(eval)),
            LengthOperator::Higher => suit_length(suit, count, prev + 1, op, Some(eval)),
            LengthOperator::Equal => suit_length(suit, count, prev, op, Some(eval)),
        }
    } else {
        // If this is the first inquiry about this suit, we return whether we have 4 or
        // more cards in it
        suit_length(suit, count, 4, LengthOperator::Higher, None)
    }
}

/// Appends the appropriate [AuctionTurn] to the auction for a [Bid] from a
/// given [Bidder]
pub fn append_bid_response(game: &mut GameData, bidder: Bidder, bid: Bid) {
    let response = match bid {
        Bid::Query => query_bid_response(game, bidder),
        Bid::Suit(suit) => suit_bid_response(game, bidder, suit),
        Bid::Pass => BidResponse::Pass,
    };

    game.auction.bids_mut(bidder).push(AuctionTurn { bid, response });
}

/// Mutates the provided [GameData] to apply the user's [Bid].
pub fn resolve_bid_action(game: &mut GameData, agent: &dyn Agent, bid: Bid) -> Result<()> {
    match next_to_bid(&game.auction) {
        Some(bidder) if game.auction.position(bidder) == Position::User => {
            append_bid_response(game, bidder, bid);
            Ok(())
        }
        _ => Err(anyhow!("Not the user's turn")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::test_helpers,
        model::{
            bidding::{AuctionTurn, BidResponse, HandBalance, HandEvaluation},
            primitives::Suit,
        },
    };

    #[test]
    fn test_has_passed() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(has_passed(&g.auction, Bidder::First), false);
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, response: BidResponse::Pass });
        assert_eq!(has_passed(&g.auction, Bidder::First), true);
        assert_eq!(has_passed(&g.auction, Bidder::Second), false);
    }

    #[test]
    fn test_is_completed() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(is_completed(&g.auction), false);
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, response: BidResponse::Pass });
        assert_eq!(is_completed(&g.auction), false);
        g.auction.second_bids.push(AuctionTurn { bid: Bid::Pass, response: BidResponse::Pass });
        assert_eq!(is_completed(&g.auction), true);
    }

    #[test]
    fn test_next_to_bid() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::First));
        g.auction.first_bids.push(AuctionTurn {
            bid: Bid::Query,
            response: BidResponse::HandEvaluation(HandEvaluation::Poor),
        });
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::Second));
        g.auction.second_bids.push(AuctionTurn {
            bid: Bid::Query,
            response: BidResponse::BestSuit(HandBalance::Balanced, Suit::Hearts),
        });
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::First));
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, response: BidResponse::Pass });
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::Second));
        g.auction.second_bids.push(AuctionTurn { bid: Bid::Pass, response: BidResponse::Pass });
        assert_eq!(next_to_bid(&g.auction), None);
    }

    #[test]
    fn test_bids_of_type() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(bids_of_type(&g.auction, Bidder::First, Bid::Suit(Suit::Hearts)), 0);
        g.auction
            .first_bids
            .push(AuctionTurn { bid: Bid::Suit(Suit::Hearts), response: BidResponse::Pass });
        g.auction
            .first_bids
            .push(AuctionTurn { bid: Bid::Suit(Suit::Diamonds), response: BidResponse::Pass });
        assert_eq!(bids_of_type(&g.auction, Bidder::First, Bid::Suit(Suit::Hearts)), 1);
    }

    #[test]
    fn test_hand_evaluation() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            hand_evaluation(g.hand(Position::User), None),
            (HandEvaluation::Fair, SuitCounts { diamonds: 0, clubs: 5, hearts: 4, spades: 4 })
        );
        assert_eq!(
            hand_evaluation(g.hand(Position::User), Some(Suit::Clubs)).0,
            HandEvaluation::Excellent
        );
        assert_eq!(
            hand_evaluation(g.hand(Position::User), Some(Suit::Diamonds)).0,
            HandEvaluation::Fair
        );

        // Left:  ♦2 ♦3 ♦9 ♦10 ♦Q ♣4 ♣8 ♥5 ♥8 ♥K ♠3 ♠6 ♠A
        assert_eq!(
            hand_evaluation(g.hand(Position::Left), None),
            (HandEvaluation::Poor, SuitCounts { diamonds: 5, clubs: 2, hearts: 3, spades: 3 })
        );
        assert_eq!(
            hand_evaluation(g.hand(Position::Left), Some(Suit::Diamonds)).0,
            HandEvaluation::Fair
        );
        assert_eq!(
            hand_evaluation(g.hand(Position::Left), Some(Suit::Clubs)).0,
            HandEvaluation::Poor
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            hand_evaluation(g.hand(Position::Dummy), None),
            (HandEvaluation::Poor, SuitCounts { diamonds: 4, clubs: 2, hearts: 4, spades: 3 })
        );

        // Right: ♦4 ♦5 ♦J ♦A ♣3 ♣7 ♣J ♣Q ♥2 ♥3 ♠9 ♠J ♠Q
        assert_eq!(
            hand_evaluation(g.hand(Position::Right), None),
            (HandEvaluation::Fair, SuitCounts { diamonds: 4, clubs: 4, hearts: 2, spades: 3 })
        );
    }

    fn user_bid_response(bid: Bid, previous: Vec<AuctionTurn>) -> BidResponse {
        let mut g = test_helpers::create_test_bid_phase();
        g.auction.first_bids = previous;
        append_bid_response(&mut g, Bidder::First, bid);
        g.auction.first_bids.last().expect("Expected response").response
    }

    #[test]
    fn test_suit_bid_responses() {
        // Dummy has 2 clubs & 4 hearts
        let lt4 = BidResponse::SuitLength(Suit::Clubs, 4, LengthOperator::Lower, None);
        assert_eq!(user_bid_response(Bid::Suit(Suit::Clubs), vec![]), lt4);

        let lt3 = BidResponse::SuitLength(
            Suit::Clubs,
            3,
            LengthOperator::Lower,
            Some(HandEvaluation::Poor),
        );
        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Clubs),
                vec![AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt4 }]
            ),
            lt3
        );

        let lt2 = BidResponse::SuitLength(
            Suit::Clubs,
            2,
            LengthOperator::Lower,
            Some(HandEvaluation::Poor),
        );
        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Clubs),
                vec![
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt4 },
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt3 }
                ]
            ),
            lt2
        );

        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Clubs),
                vec![
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt4 },
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt3 },
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt2 }
                ]
            ),
            BidResponse::SuitLength(
                Suit::Clubs,
                2,
                LengthOperator::Equal,
                Some(HandEvaluation::Poor)
            )
        );

        let gt4 = BidResponse::SuitLength(Suit::Hearts, 4, LengthOperator::Higher, None);
        assert_eq!(user_bid_response(Bid::Suit(Suit::Hearts), vec![]), gt4);

        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Hearts),
                vec![AuctionTurn { bid: Bid::Suit(Suit::Hearts), response: gt4 }]
            ),
            BidResponse::SuitLength(
                Suit::Hearts,
                4,
                LengthOperator::Equal,
                Some(HandEvaluation::Fair)
            )
        );

        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Hearts),
                vec![AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt4 }]
            ),
            gt4
        );

        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Hearts),
                vec![
                    AuctionTurn { bid: Bid::Suit(Suit::Clubs), response: lt4 },
                    AuctionTurn { bid: Bid::Suit(Suit::Hearts), response: gt4 }
                ]
            ),
            BidResponse::SuitLength(
                Suit::Hearts,
                4,
                LengthOperator::Equal,
                Some(HandEvaluation::Fair)
            )
        );

        assert_eq!(
            user_bid_response(
                Bid::Suit(Suit::Hearts),
                vec![AuctionTurn {
                    bid: Bid::Query,
                    response: BidResponse::HandEvaluation(HandEvaluation::Excellent)
                }]
            ),
            gt4
        );
    }
}
