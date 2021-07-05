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

//! Defines a simple agent which uses deterministic heuristics to decide on its
//! game actions

use std::{cmp::Ordering, collections::HashSet};

use strum::IntoEnumIterator;

use crate::{
    agents::agent::Agent,
    game::{
        bidding_phase::{self, HandScore},
        play_phase,
    },
    model::{
        bidding::{Bid, BidResponse, Bidder, LengthOperator},
        game::{GameData, PlayPhaseData},
        primitives::{Card, CardId, Position, Suit},
    },
};

#[derive(Debug)]
pub struct HeuristicAgent;

impl Agent for HeuristicAgent {
    /// Selects a bid, using a simple bid priority system. Aborts bidding if
    /// predicted combined points is below a threshold.
    fn select_bid(&self, game: &GameData, bidder: Bidder) -> Bid {
        let responses = game
            .auction
            .bids(bidder)
            .iter()
            .rev()
            .flat_map(|turn| &turn.responses)
            .copied()
            .collect::<Vec<_>>();
        let hand = game.hand(game.auction.position(bidder));
        let hand_score = bidding_phase::hand_score(hand);
        let trump_fit = find_trump_fit(hand_score, &responses);

        let points = predicted_combined_points(hand_score, &responses, trump_fit);
        match game.auction.bid_number {
            6 if points < 16 => Bid::Pass,
            7 if points < 24 => Bid::Pass,
            8 if points < 26 => Bid::Pass,
            9 if points < 28 => Bid::Pass,
            10 if points < 30 => Bid::Pass,
            11 if points < 32 => Bid::Pass,
            12 if points < 34 => Bid::Pass,
            _ => highest_priority_bid(hand_score, &responses, trump_fit),
        }
    }

    /// Wins the trick if possible & partner is not already winning, otherwise
    /// discards
    fn select_play(&self, data: &PlayPhaseData, position: Position) -> usize {
        let winner =
            if play_phase::trick_winner(data).map_or(true, |(w, _)| w != position.partner()) {
                find_winning_card(data, position)
            } else {
                None
            };

        winner.unwrap_or_else(|| find_discard(data, position))
    }
}

/// Checks if agent & partner have an 8-card fit between them
fn find_trump_fit(hand_score: HandScore, responses: &[BidResponse]) -> Option<Suit> {
    responses
        .iter()
        .filter_map(|response| match *response {
            BidResponse::SuitLength(suit, size, op)
                if op != LengthOperator::Lte && size + hand_score.counts.get(suit) >= 8 =>
            {
                Some(suit)
            }
            BidResponse::LongestSuit(suit) if 5 + hand_score.counts.get(suit) >= 8 => {
                // 65% chance they have 5+ cards
                Some(suit)
            }
            _ => None,
        })
        .max_by_key(|suit| hand_score.scores.get(*suit))
}

/// Predicts the number of points available to the partnership
fn predicted_combined_points(
    hand_score: HandScore,
    responses: &[BidResponse],
    trump: Option<Suit>,
) -> usize {
    responses
        .iter()
        .filter_map(|r| match r {
            BidResponse::HandEvaluation(rating, Some(t))
                if trump.map_or(false, |trump| trump == *t) =>
            {
                Some(
                    rating.approximate_points() +
                        bidding_phase::evaluate_hand(hand_score, Some(*t)),
                )
            }
            BidResponse::HandEvaluation(rating, _) => {
                Some(rating.approximate_points() + hand_score.scores.sum())
            }
            _ => None,
        })
        .max()
        .unwrap_or(hand_score.scores.sum() + 10)
}

/// Searches for the longest/strongest suit to bid which we currently have no
/// information about, returning a triple of (count, score, suit).
fn prioritized_suit_bid(
    hand_score: HandScore,
    responses: &[BidResponse],
) -> Option<(usize, usize, Suit)> {
    let known = responses
        .iter()
        .filter_map(|r| match *r {
            BidResponse::SuitLength(s, _, _) |
            BidResponse::HandEvaluation(_, Some(s)) |
            BidResponse::LongestSuit(s) |
            BidResponse::WeakestSuit(s) => Some(s),
            _ => None,
        })
        .collect::<HashSet<_>>();
    Suit::iter()
        .filter_map(|s| {
            if known.contains(&s) {
                None
            } else {
                Some((hand_score.counts.get(s), hand_score.scores.get(s), s))
            }
        })
        .max_by(|(c1, s1, _), (c2, s2, _)| c1.cmp(c2).then(s1.cmp(s2)))
}

/// Counts [BidResponse::HandEvaluation] responses
fn evaluation_count(responses: &[BidResponse]) -> usize {
    responses.iter().filter(|r| matches!(r, BidResponse::HandEvaluation(_, _))).count()
}

/// Picks a bid to make based on a priority order
fn highest_priority_bid(
    hand_score: HandScore,
    responses: &[BidResponse],
    trump: Option<Suit>,
) -> Bid {
    if let Some(s) = trump {
        // Priority #1: Raise trump fit
        Bid::Suit(s)
    } else if let Some((count, score, suit)) = prioritized_suit_bid(hand_score, responses) {
        if count >= 5 {
            // Priority #2: 5+ card suit we have no information about
            Bid::Suit(suit)
        } else if evaluation_count(responses) == 0 {
            // Priority #3: First 'Query' bid
            Bid::Query
        } else if count >= 4 && score >= 3 {
            // Priority #4: Strong 4-card suit
            Bid::Suit(suit)
        } else {
            // Priority #5: Additional 'Query' bids
            Bid::Query
        }
    } else {
        Bid::Query
    }
}

/// Returns the position of the highest power card in hand which can win the
/// current trick, if any, returning higher rank cards later in hand in the
/// event of a tie
fn find_winning_card(game: &PlayPhaseData, position: Position) -> Option<usize> {
    play_phase::winning_plays(game, position)
        .max_by(|(_, a), (_, b)| {
            play_phase::compare_card_power(game, *a, *b).then(a.rank.cmp(&b.rank))
        })
        .map(|(index, _)| index)
}

/// Discards the lowest-ranking card, either following suit or throwing off
fn find_discard(game: &PlayPhaseData, position: Position) -> usize {
    play_phase::legal_plays(game, position)
        .min_by(|(_, a), (_, b)| {
            play_phase::compare_card_power(game, *a, *b).then(a.rank.cmp(&b.rank))
        })
        .map(|(index, _)| index)
        .expect("No legal plays")
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::{
        game::test_helpers,
        model::primitives::{Card, Rank, Suit},
    };

    #[test]
    fn test_select_play() {
        let agent = HeuristicAgent {};
        let mut g = test_helpers::create_test_play_phase();

        let p1 = agent.select_play(&g, Position::User);
        assert_eq!(g.game.hands.user_hand[p1], Card::new(Suit::Hearts, Rank::Ace));
        play_phase::play_card(&mut g, CardId::new(Position::User, p1));
        let p2 = agent.select_play(&g, Position::Left);
        assert_eq!(g.game.hands.left_opponent_hand[p2], Card::new(Suit::Hearts, Rank::Five));
        play_phase::play_card(&mut g, CardId::new(Position::Left, p2));
        let p3 = agent.select_play(&g, Position::Dummy);
        assert_eq!(g.game.hands.dummy_hand[p3], Card::new(Suit::Hearts, Rank::Four));
        play_phase::play_card(&mut g, CardId::new(Position::Dummy, p3));
        let p4 = agent.select_play(&g, Position::Right);
        assert_eq!(g.game.hands.right_opponet_hand[p4], Card::new(Suit::Hearts, Rank::Two));
    }

    #[test]
    fn test_select_bid() {
        let agent = HeuristicAgent {};
        // Hand (User, 11): ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
        // Hand (Dummy, 9): ♦6 ♦7 ♦8 ♦K ♣5 ♣K ♥4 ♥7 ♥J ♥Q ♠4 ♠5 ♠10
        let mut g = test_helpers::create_test_bid_phase();
        let b1 = agent.select_bid(&g, Bidder::First);
        assert_eq!(Bid::Suit(Suit::Clubs), b1); // Has 5 Clubs. Response: <= 3 Clubs
        bidding_phase::append_bid_response(&mut g, Bidder::First, b1);

        // Hand (Right, 11): ♦4 ♦5 ♦J ♦A ♣3 ♣7 ♣J ♣Q ♥2 ♥3 ♠9 ♠J ♠Q
        // Hand (Left, 9): ♦2 ♦3 ♦9 ♦10 ♦Q ♣4 ♣8 ♥5 ♥8 ♥K ♠3 ♠6 ♠A
        let b2 = agent.select_bid(&g, Bidder::Second);
        assert_eq!(Bid::Query, b2); // Has no 5-card suit. Response: Poor
        bidding_phase::append_bid_response(&mut g, Bidder::Second, b2);

        g.auction.bid_number += 1;

        let b3 = agent.select_bid(&g, Bidder::First);
        assert_eq!(Bid::Pass, b3);

        let b4 = agent.select_bid(&g, Bidder::Second);
        assert_eq!(Bid::Pass, b4);
    }
}
