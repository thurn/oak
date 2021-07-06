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

//! Responsible for dealing cards & creating new Game instances

use std::{collections::HashMap, slice::ChunksExact};

use rand::{prelude::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg64;
use strum::IntoEnumIterator;

use crate::model::{
    bidding::Auction,
    game::{Debug, GameData, Hands, Trick},
    primitives::{Card, Position, Rank, Suit},
};

/// Creates a new [GameData] dealing hands to the four positions
pub fn new_game(rng: &mut impl Rng, first: Position, second: Position) -> GameData {
    let mut cards = Vec::new();
    for suit in Suit::iter() {
        for rank in Rank::iter() {
            cards.push(Card { suit, rank })
        }
    }
    cards.shuffle(rng);
    let mut chunks = cards.chunks_exact(13);

    fn build_hand(chunks: &mut ChunksExact<Card>) -> Vec<Card> {
        let mut hand = chunks.next().expect("Invalid deck size").to_vec();
        hand.sort_unstable();
        hand
    }

    GameData {
        hands: Hands {
            user_hand: build_hand(&mut chunks),
            left_opponent_hand: build_hand(&mut chunks),
            dummy_hand: build_hand(&mut chunks),
            right_opponet_hand: build_hand(&mut chunks),
        },
        auction: Auction { bid_number: 6, first, first_bids: vec![], second, second_bids: vec![] },
        debug: Debug { show_hidden_cards: true },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::test_helpers;

    #[test]
    fn test_new_game() {
        let g = new_game(&mut Pcg64::seed_from_u64(17), Position::User, Position::Left);
        assert_eq!(g.hands.user_hand.len(), 13);
        assert_eq!(g.hands.left_opponent_hand.len(), 13);
        assert_eq!(g.hands.dummy_hand.len(), 13);
        assert_eq!(g.hands.right_opponet_hand.len(), 13);
        assert_eq!(g.hands.user_hand[0], test_helpers::USER_CARD_0)
    }
}
