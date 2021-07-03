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

//! Defines a simple agent which uses fixed heuristics to decide on its game
//! actions

use std::cmp::Ordering;

use crate::{
    agents::agent::Agent,
    game::play_phase,
    model::{
        game::PlayPhaseData,
        primitives::{CardId, Position},
    },
};

#[derive(Debug)]
pub struct HeuristicAgent;

impl Agent for HeuristicAgent {
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

/// Returns the position of the highest power card in hand which can win the
/// current trick, if any, returning higher rank cards later in hand in the
/// event of a tie
fn find_winning_card(game: &PlayPhaseData, position: Position) -> Option<usize> {
    play_phase::winning_plays(game, position)
        .max_by(|(_, a), (_, b)| match play_phase::compare_card_power(game, *a, *b) {
            Ordering::Equal => a.rank.cmp(&b.rank),
            ordering => ordering,
        })
        .map(|(index, _)| index)
}

/// Discards the lowest-ranking card, either following suit or throwing off
fn find_discard(game: &PlayPhaseData, position: Position) -> usize {
    play_phase::legal_plays(game, position)
        .min_by(|(_, a), (_, b)| match play_phase::compare_card_power(game, *a, *b) {
            Ordering::Equal => a.rank.cmp(&b.rank),
            ordering => ordering,
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
}
