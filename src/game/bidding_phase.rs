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

use std::{cmp::Ordering, mem};

use anyhow::{anyhow, Result};
use strum::IntoEnumIterator;

use crate::{
    agents::agent::Agent,
    model::{
        bidding::{
            Auction,
            AuctionTurn,
            Bid,
            BidResponse,
            Bidder,
            HandBalance,
            HandRating,
            LengthOperator,
        },
        game::{Contract, GameData, GamePhase, PlayPhaseData, Trick},
        primitives::{Card, Position, Rank, Suit, SuitData},
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

/// Returned from [hand_score] with the suit count & suit high card points
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct HandScore {
    pub counts: SuitData,
    pub scores: SuitData,
}

/// Returns a (count, score) [SuitData] pair for this hand representing the
/// count of cards in each suit and the high card point score for each suit
pub fn hand_score(hand: &[Card]) -> HandScore {
    let mut counts = SuitData::default();
    let mut scores = SuitData::default();

    for card in hand {
        *scores.get_mut(card.suit) = scores.get(card.suit) +
            match card.rank {
                Rank::Ace => 4,
                Rank::King => 3,
                Rank::Queen => 2,
                Rank::Jack => 1,
                _ => 0,
            };
        counts.increment(card.suit);
    }

    HandScore { counts, scores }
}

/// Computes a score which evaluates the strength of a hand, adding short suit
// points for a known trump suit if provided.
pub fn evaluate_hand(hand_score: HandScore, trump_suit: Option<Suit>) -> usize {
    trump_suit.map_or(hand_score.scores.sum(), |trump| {
        let short_suit_points = Suit::iter()
            .map(|suit| match hand_score.counts.get(suit) {
                _ if suit == trump => 0,
                0 => 5,
                1 => 3,
                2 => 1,
                _ => 0,
            })
            .sum::<usize>();
        hand_score.scores.sum() + short_suit_points
    })
}

/// Wraps the result of [evaluate_hand] in a [BidResponse::HandEvaluation]
pub fn hand_evaluation(hand_score: HandScore, trump_suit: Option<Suit>) -> BidResponse {
    BidResponse::HandEvaluation(HandRating::new(evaluate_hand(hand_score, trump_suit)), trump_suit)
}

/// Produces a [BidResponse::HandBalance] for this hand
pub fn hand_balance(hand_score: HandScore) -> BidResponse {
    BidResponse::HandBalance(
        if !Suit::iter().any(|suit| hand_score.counts.get(suit) <= 1) &&
            Suit::iter().filter(|suit| hand_score.counts.get(*suit) <= 2).count() <= 1
        {
            HandBalance::Balanced // Balanced -- at most one doubleton, no
                                  // singletons or voids
        } else {
            HandBalance::Unbalanced
        },
    )
}

/// Produces a [BidResponse::LongestSuit] response for this hand, identifying
/// the longest and strongest suit
pub fn longest_suit(hand_score: HandScore) -> BidResponse {
    let HandScore { counts, scores } = hand_score;

    BidResponse::LongestSuit(
        Suit::iter()
            .max_by(|x, y| {
                counts.get(*x).cmp(&counts.get(*y)).then(scores.get(*x).cmp(&scores.get(*y)))
            })
            .expect("Suit::iter() cannot be empty"),
    )
}

/// Produces a [BidResponse::WeakestSuit] response for this hand
pub fn weakest_suit(hand_score: HandScore) -> BidResponse {
    let HandScore { counts, scores } = hand_score;
    BidResponse::WeakestSuit(
        Suit::iter()
            .min_by(|x, y| {
                scores.get(*x).cmp(&scores.get(*y)).then(counts.get(*x).cmp(&counts.get(*y)))
            })
            .expect("Suit::iter() cannot be empty"),
    )
}

/// Produces a [BidResponse::RankCount] for a given [Rank] in this hand
pub fn rank_count(hand: &[Card], rank: Rank) -> BidResponse {
    BidResponse::RankCount(rank, hand.iter().filter(|card| card.rank == rank).count())
}

/// Returns the [BidResponse] for a [Bid::Query] bid
pub fn query_bid_response(game: &GameData, bidder: Bidder) -> Vec<BidResponse> {
    let hand = game.hand(game.auction.position(bidder).partner());
    let hand_score = hand_score(hand);
    match game.auction.bids(bidder).iter().filter(|turn| turn.bid == Bid::Query).count() {
        0 => vec![hand_evaluation(hand_score, None)],
        1 => vec![hand_balance(hand_score), longest_suit(hand_score)],
        2 => vec![weakest_suit(hand_score)],
        3 => vec![rank_count(hand, Rank::Ace)],
        4 => vec![rank_count(hand, Rank::King)],
        5 => vec![rank_count(hand, Rank::Queen)],
        _ => vec![BidResponse::Pass],
    }
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
        .flat_map(|turn| &turn.responses)
        .filter_map(|response| match response {
            BidResponse::SuitLength(s, len, op) if *s == suit => Some((*len, *op)),
            _ => None,
        })
        .next()
}

/// Construct a new [BidResponse::SuitLength] by comparing the length in hand
/// with a target, returning 'bias' if they are equal.
pub fn suit_length(
    suit: Suit,
    have: usize,
    constraint: usize,
    bias: LengthOperator,
) -> BidResponse {
    let op = match have.cmp(&constraint) {
        Ordering::Less => LengthOperator::Lte,
        Ordering::Equal => bias,
        Ordering::Greater => LengthOperator::Gte,
    };

    BidResponse::SuitLength(suit, constraint, op)
}

/// Returns a [BidResponse::SuitLength] for a [Bid::Suit] bid
pub fn suit_bid_response(game: &GameData, bidder: Bidder, suit: Suit) -> Vec<BidResponse> {
    let score = hand_score(game.hand(game.auction.position(bidder).partner()));
    let count = score.counts.get(suit);

    if let Some((prev, op)) = previous_suit_response(game, bidder, suit) {
        // If we've previously responded for this suit, we increment or decrement our
        // response by 1. If this would reveal the exact count, we provide an EqualTo
        // response.
        let length = match op {
            _ if prev == count => suit_length(suit, count, prev, LengthOperator::Equal),
            LengthOperator::Lte => suit_length(suit, count, prev - 1, op),
            LengthOperator::Gte => suit_length(suit, count, prev + 1, op),
            LengthOperator::Equal => suit_length(suit, count, prev, op),
        };

        // We also include a hand evaluation updated based on this trump suit
        vec![length, hand_evaluation(score, Some(suit))]
    } else {
        // If this is the first response for a given suit, we return
        // 1) whether we have >= 3 of that suit if this is the opening bid, or
        // 2) whether we have >= 4 of that suit otherwise
        let target = if game.auction.bids(bidder).is_empty() { 3 } else { 4 };
        vec![suit_length(suit, count, target, LengthOperator::Gte)]
    }
}

/// Appends the appropriate [AuctionTurn] to the auction for a [Bid] from a
/// given [Bidder], incrementing the bid number if needed
pub fn append_bid_response(game: &mut GameData, bidder: Bidder, bid: Bid) {
    let responses = match bid {
        Bid::Query => query_bid_response(game, bidder),
        Bid::Suit(suit) => suit_bid_response(game, bidder, suit),
        Bid::Pass => vec![BidResponse::Pass],
    };

    game.auction.bids_mut(bidder).push(AuctionTurn { bid, responses });

    if game.auction.first_bids.len() == game.auction.second_bids.len() ||
        has_passed(&game.auction, bidder.opposite())
    {
        game.auction.bid_number += 1;
    }
}

fn find_contract(game: &GameData, declarer: Bidder) -> Contract {
    Contract {
        trump: game
            .auction
            .bids(declarer)
            .iter()
            .rev()
            .filter(|turn| !matches!(turn, AuctionTurn { bid: Bid::Pass, .. }))
            .map(|bid| match bid {
                AuctionTurn { bid: Bid::Suit(suit), .. } => Some(*suit),
                _ => None,
            })
            .next()
            .flatten(),
        tricks: game.auction.bid_number - 1, // Final round of bidding does not count
        declarer: game.auction.position(declarer),
    }
}

pub fn advance_to_play_phase(phase: &mut GamePhase) -> Result<()> {
    // Temporarily set the phase to 'Starting' while renovations are ongoing
    match mem::replace(phase, GamePhase::Starting) {
        GamePhase::Auction(game) => {
            let declarer = if game.auction.first_bids.len() > game.auction.second_bids.len() {
                Bidder::First
            } else {
                Bidder::Second
            };

            let trick = Trick::new(game.auction.position(declarer));
            let contract = find_contract(&game, declarer);

            *phase = GamePhase::Playing(PlayPhaseData { game, trick, contract });
            Ok(())
        }
        _ => Err(anyhow!("Not in the Auction phase")),
    }
}

/// Mutates the provided [GamePhase] to apply the user's [Bid], transitioning it
/// to [GamePhase::Playing] if the auction is now completed.
pub fn resolve_bid_action(phase: &mut GamePhase, agent: &dyn Agent, bid: Bid) -> Result<()> {
    match phase {
        GamePhase::Auction(ref mut game) => match next_to_bid(&game.auction) {
            Some(bidder) if game.auction.position(bidder) == Position::User => {
                append_bid_response(game, bidder, bid);

                let opposite = bidder.opposite();
                if next_to_bid(&game.auction) == Some(opposite) {
                    append_bid_response(game, opposite, agent.select_bid(game, opposite))
                }

                if is_completed(&game.auction) {
                    advance_to_play_phase(phase)
                } else {
                    Ok(())
                }
            }
            _ => Err(anyhow!("Not the user's turn")),
        },
        _ => Err(anyhow!("Can only bid during the Auction phase")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agents::{agent, constant::ConstantAgent},
        game::test_helpers,
        model::{
            bidding::{AuctionTurn, BidResponse, HandBalance, HandRating},
            primitives::Suit,
        },
    };

    #[test]
    fn test_has_passed() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(has_passed(&g.auction, Bidder::First), false);
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, responses: vec![] });
        assert_eq!(has_passed(&g.auction, Bidder::First), true);
        assert_eq!(has_passed(&g.auction, Bidder::Second), false);
    }

    #[test]
    fn test_is_completed() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(is_completed(&g.auction), false);
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, responses: vec![] });
        assert_eq!(is_completed(&g.auction), false);
        g.auction.second_bids.push(AuctionTurn { bid: Bid::Pass, responses: vec![] });
        assert_eq!(is_completed(&g.auction), true);
    }

    #[test]
    fn test_next_to_bid() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::First));
        g.auction
            .first_bids
            .push(AuctionTurn::query(BidResponse::HandEvaluation(HandRating::Poor, None)));
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::Second));
        g.auction.second_bids.push(AuctionTurn::query(BidResponse::LongestSuit(
            // HandBalance::Balanced,
            Suit::Hearts,
        )));
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::First));
        g.auction.first_bids.push(AuctionTurn { bid: Bid::Pass, responses: vec![] });
        assert_eq!(next_to_bid(&g.auction), Some(Bidder::Second));
        g.auction.second_bids.push(AuctionTurn { bid: Bid::Pass, responses: vec![] });
        assert_eq!(next_to_bid(&g.auction), None);
    }

    #[test]
    fn test_bids_of_type() {
        let mut g = test_helpers::create_test_bid_phase();
        assert_eq!(bids_of_type(&g.auction, Bidder::First, Bid::Suit(Suit::Hearts)), 0);
        g.auction.first_bids.push(AuctionTurn::suit(Suit::Hearts, BidResponse::Pass));
        g.auction.first_bids.push(AuctionTurn::suit(Suit::Diamonds, BidResponse::Pass));
        assert_eq!(bids_of_type(&g.auction, Bidder::First, Bid::Suit(Suit::Hearts)), 1);
    }

    #[test]
    fn test_hand_score() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            hand_score(g.hand(Position::User)),
            HandScore {
                counts: SuitData { diamonds: 0, clubs: 5, hearts: 4, spades: 4 },
                scores: SuitData { diamonds: 0, clubs: 4, hearts: 4, spades: 3 }
            }
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            hand_score(g.hand(Position::Dummy)),
            HandScore {
                counts: SuitData { diamonds: 4, clubs: 2, hearts: 4, spades: 3 },
                scores: SuitData { diamonds: 3, clubs: 3, hearts: 3, spades: 0 }
            }
        );
    }

    #[test]
    fn test_hand_evaluation() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            hand_evaluation(hand_score(g.hand(Position::User)), Some(Suit::Clubs)),
            BidResponse::HandEvaluation(HandRating::Excellent, Some(Suit::Clubs))
        );
        assert_eq!(
            hand_evaluation(hand_score(g.hand(Position::User)), Some(Suit::Diamonds)),
            BidResponse::HandEvaluation(HandRating::Fair, Some(Suit::Diamonds))
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            hand_evaluation(hand_score(g.hand(Position::Dummy)), Some(Suit::Clubs)),
            BidResponse::HandEvaluation(HandRating::Poor, Some(Suit::Clubs))
        );
        assert_eq!(
            hand_evaluation(hand_score(g.hand(Position::Dummy)), Some(Suit::Diamonds)),
            BidResponse::HandEvaluation(HandRating::Fair, Some(Suit::Diamonds))
        );
    }

    #[test]
    fn test_longest_suit() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            longest_suit(hand_score(g.hand(Position::User))),
            // HandBalance::Unbalanced
            BidResponse::LongestSuit(Suit::Clubs)
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            longest_suit(hand_score(g.hand(Position::Dummy))),
            // HandBalance::Balanced
            BidResponse::LongestSuit(Suit::Hearts)
        );
    }

    #[test]
    fn test_weakest_suit() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            weakest_suit(hand_score(g.hand(Position::User))),
            BidResponse::WeakestSuit(Suit::Diamonds)
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            weakest_suit(hand_score(g.hand(Position::Dummy))),
            BidResponse::WeakestSuit(Suit::Spades)
        );
    }

    #[test]
    fn test_rank_count() {
        let g = test_helpers::create_test_bid_phase();
        // User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        assert_eq!(
            rank_count(g.hand(Position::User), Rank::Ace),
            BidResponse::RankCount(Rank::Ace, 2)
        );
        assert_eq!(
            rank_count(g.hand(Position::User), Rank::King),
            BidResponse::RankCount(Rank::King, 1)
        );

        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        assert_eq!(
            rank_count(g.hand(Position::Dummy), Rank::Ace),
            BidResponse::RankCount(Rank::Ace, 0)
        );
        assert_eq!(
            rank_count(g.hand(Position::Dummy), Rank::Queen),
            BidResponse::RankCount(Rank::Queen, 1)
        );
    }

    fn get_dummy_response(bid: Bid, previous: Vec<AuctionTurn>) -> Vec<BidResponse> {
        let mut g = test_helpers::create_test_bid_phase();
        g.auction.first_bids = previous;
        append_bid_response(&mut g, Bidder::First, bid);
        g.auction.first_bids.last().expect("Expected response").responses.clone()
    }

    #[test]
    fn test_query_bid_responses() {
        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        fn query(response: BidResponse) -> AuctionTurn {
            AuctionTurn { bid: Bid::Query, responses: vec![response] }
        }

        let eval = BidResponse::HandEvaluation(HandRating::Poor, None);
        assert_eq!(get_dummy_response(Bid::Query, vec![]), vec![eval]);

        let longest = BidResponse::LongestSuit(Suit::Hearts);
        assert_eq!(
            get_dummy_response(Bid::Query, vec![query(eval)]),
            vec![BidResponse::HandBalance(HandBalance::Balanced), longest]
        );

        let weakest = BidResponse::WeakestSuit(Suit::Spades);
        assert_eq!(
            get_dummy_response(Bid::Query, vec![query(eval), query(longest)]),
            vec![weakest]
        );

        let aces = BidResponse::RankCount(Rank::Ace, 0);
        assert_eq!(
            get_dummy_response(Bid::Query, vec![query(eval), query(longest), query(weakest)]),
            vec![aces]
        );

        let kings = BidResponse::RankCount(Rank::King, 2);
        assert_eq!(
            get_dummy_response(
                Bid::Query,
                vec![query(eval), query(longest), query(weakest), query(aces)]
            ),
            vec![kings]
        );

        let queens = BidResponse::RankCount(Rank::Queen, 1);
        assert_eq!(
            get_dummy_response(
                Bid::Query,
                vec![query(eval), query(longest), query(weakest), query(aces), query(kings)]
            ),
            vec![queens]
        );

        assert_eq!(
            get_dummy_response(
                Bid::Query,
                vec![
                    query(eval),
                    query(longest),
                    query(weakest),
                    query(aces),
                    query(kings),
                    query(queens)
                ]
            ),
            vec![BidResponse::Pass]
        );
    }

    #[test]
    fn test_suit_bid_responses() {
        // Dummy: ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        let lt3c = BidResponse::SuitLength(Suit::Clubs, 3, LengthOperator::Lte);
        assert_eq!(get_dummy_response(Bid::Suit(Suit::Clubs), vec![]), vec![lt3c]);

        let lt2c = BidResponse::SuitLength(Suit::Clubs, 2, LengthOperator::Lte);
        assert_eq!(
            get_dummy_response(Bid::Suit(Suit::Clubs), vec![AuctionTurn::suit(Suit::Clubs, lt3c)]),
            vec![lt2c, BidResponse::HandEvaluation(HandRating::Poor, Some(Suit::Clubs))]
        );

        assert_eq!(
            get_dummy_response(
                Bid::Suit(Suit::Clubs),
                vec![AuctionTurn::suit(Suit::Clubs, lt3c), AuctionTurn::suit(Suit::Clubs, lt2c)]
            ),
            vec![
                BidResponse::SuitLength(Suit::Clubs, 2, LengthOperator::Equal),
                BidResponse::HandEvaluation(HandRating::Poor, Some(Suit::Clubs))
            ]
        );

        let gt3s = BidResponse::SuitLength(Suit::Spades, 3, LengthOperator::Gte);
        assert_eq!(get_dummy_response(Bid::Suit(Suit::Spades), vec![]), vec![gt3s]);

        // Some(HandEvaluation::Fair)
        assert_eq!(
            get_dummy_response(
                Bid::Suit(Suit::Spades),
                vec![AuctionTurn::suit(Suit::Spades, gt3s)]
            ),
            vec![
                BidResponse::SuitLength(Suit::Spades, 3, LengthOperator::Equal),
                BidResponse::HandEvaluation(HandRating::Fair, Some(Suit::Spades))
            ]
        );

        assert_eq!(
            get_dummy_response(Bid::Suit(Suit::Spades), vec![AuctionTurn::suit(Suit::Clubs, lt3c)]),
            vec![BidResponse::SuitLength(Suit::Spades, 4, LengthOperator::Lte)]
        );

        assert_eq!(
            get_dummy_response(
                Bid::Suit(Suit::Hearts),
                vec![
                    AuctionTurn::suit(Suit::Clubs, lt3c),
                    AuctionTurn::suit(
                        Suit::Hearts,
                        BidResponse::SuitLength(Suit::Hearts, 4, LengthOperator::Gte)
                    )
                ]
            ),
            vec![
                BidResponse::SuitLength(Suit::Hearts, 4, LengthOperator::Equal),
                BidResponse::HandEvaluation(HandRating::Fair, Some(Suit::Hearts))
            ]
        );
    }

    #[test]
    fn test_pass_bid_responses() {
        assert_eq!(get_dummy_response(Bid::Pass, vec![]), vec![BidResponse::Pass]);

        assert_eq!(
            get_dummy_response(Bid::Pass, vec![AuctionTurn::query(BidResponse::Pass)]),
            vec![BidResponse::Pass]
        );
    }

    #[test]
    fn test_advance_to_play_phase() {
        fn run(
            first: Vec<AuctionTurn>,
            second: Vec<AuctionTurn>,
            round: usize,
        ) -> (Contract, Trick) {
            let agent = test_helpers::create_test_agent();
            let mut game = test_helpers::create_test_bid_phase();
            game.auction.first_bids.extend(first);
            game.auction.second_bids.extend(second);
            game.auction.bid_number = round;
            let mut phase = GamePhase::Auction(game);

            resolve_bid_action(&mut phase, &*agent, Bid::Pass).unwrap();

            if let GamePhase::Playing(data) = phase {
                (data.contract, data.trick)
            } else {
                panic!("Expected GamePhase::Playing")
            }
        }

        let pass = AuctionTurn { bid: Bid::Pass, responses: vec![BidResponse::Pass] };
        let diamonds = AuctionTurn {
            bid: Bid::Suit(Suit::Diamonds),
            responses: vec![BidResponse::SuitLength(Suit::Diamonds, 5, LengthOperator::Gte)],
        };
        let query = AuctionTurn {
            bid: Bid::Query,
            responses: vec![BidResponse::HandEvaluation(HandRating::Good, None)],
        };

        let (contract, trick) = run(vec![], vec![], 6);
        assert_eq!(contract, Contract { trump: None, tricks: 6, declarer: Position::Right });
        assert_eq!(trick, Trick::new(Position::Right));

        let (contract, trick) = run(vec![diamonds.clone()], vec![pass.clone()], 7);
        assert_eq!(
            contract,
            Contract { trump: Some(Suit::Diamonds), tricks: 7, declarer: Position::User }
        );
        assert_eq!(trick, Trick::new(Position::User));

        let (contract, trick) = run(vec![query.clone()], vec![pass.clone()], 7);
        assert_eq!(contract, Contract { trump: None, tricks: 7, declarer: Position::User });
        assert_eq!(trick, Trick::new(Position::User));
    }

    #[test]
    fn test_resolve_bid_action() {
        let g = test_helpers::create_test_bid_phase();
        let mut phase = GamePhase::Auction(g);
        fn get_game(phase: &GamePhase) -> &GameData {
            if let GamePhase::Auction(g) = phase {
                g
            } else {
                panic!("Expected a GamePhase::Auction");
            }
        }

        let agent = test_helpers::create_test_agent();
        assert!(resolve_bid_action(&mut phase, &*agent, Bid::Query).is_ok());
        assert_eq!(get_game(&phase).auction.bids(Bidder::First)[0].bid, Bid::Query);
        assert_eq!(
            get_game(&phase).auction.bids(Bidder::First)[0].responses,
            vec![BidResponse::HandEvaluation(HandRating::Poor, None)]
        );
        assert_eq!(get_game(&phase).auction.bids(Bidder::Second)[0].bid, Bid::Pass);
        assert_eq!(
            get_game(&phase).auction.bids(Bidder::Second)[0].responses,
            vec![BidResponse::Pass]
        );

        assert!(resolve_bid_action(&mut phase, &*agent, Bid::Pass).is_ok());
        if let GamePhase::Playing(data) = phase {
            assert_eq!(
                data.contract,
                Contract { trump: None, tricks: 7, declarer: Position::User }
            );
            assert_eq!(data.trick, Trick::new(Position::User))
        } else {
            panic!("Expected GamePhase::Playing");
        }
    }
}
