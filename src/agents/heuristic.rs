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

//! Defines a simple agent which uses fixed heuristics to decide on its game actions

use std::cmp::Ordering;

use crate::{
    game::play_phase,
    model::{
        game::Game,
        primitives::{CardId, Position},
    },
};

use super::agent::Agent;

struct HeuristicAgent;

impl Agent for HeuristicAgent {
    /// Wins the trick if possible & partner is not already winning, otherwise discards
    fn select_play(&self, game: &Game, position: Position) -> usize {
        let winner =
            if play_phase::trick_winner(game).map_or(true, |(w, _)| w != position.partner()) {
                find_winning_card(game, position)
            } else {
                None
            };

        winner.unwrap_or_else(|| find_discard(game, position))
    }
}

/// Returns the position of the highest power card in hand which can win the current
/// trick, if any, returning higher rank cards later in hand in the event of a tie
fn find_winning_card(game: &Game, position: Position) -> Option<usize> {
    play_phase::winning_plays(game, position)
        .max_by(
            |(_, a), (_, b)| match play_phase::compare_card_power(game, *a, *b) {
                Ordering::Equal => a.rank.cmp(&b.rank),
                ordering => ordering,
            },
        )
        .map(|(index, _)| index)
}

/// Discards the lowest-ranking card, either following suit or throwing off
fn find_discard(game: &Game, position: Position) -> usize {
    play_phase::legal_plays(game, position)
        .min_by(|(_, a), (_, b)| play_phase::compare_card_power(game, *a, *b))
        .map(|(index, _)| index)
        .expect("No cards in hand")
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        game::test_helpers,
        model::primitives::{Card, Rank, Suit},
    };

    use super::*;

    #[test]
    fn test_select_play() {
        let agent = HeuristicAgent {};
        let mut g = test_helpers::create_test_game();
        let p1 = agent.select_play(&g, Position::User);
        assert_eq!(g.user_hand[p1], Card::new(Suit::Clubs, Rank::Ace));
        play_phase::play_card(&mut g, CardId::new(Position::User, p1));
        let p2 = agent.select_play(&g, Position::Left);
        assert_eq!(g.left_opponent_hand[p2], Card::new(Suit::Clubs, Rank::Four));
        play_phase::play_card(&mut g, CardId::new(Position::Left, p2));
        let p3 = agent.select_play(&g, Position::Dummy);
        assert_eq!(g.dummy_hand[p3], Card::new(Suit::Clubs, Rank::Five));
        play_phase::play_card(&mut g, CardId::new(Position::Dummy, p3));
        let p4 = agent.select_play(&g, Position::Right);
        assert_eq!(
            g.right_opponet_hand[p4],
            Card::new(Suit::Clubs, Rank::Three)
        );
    }
}
