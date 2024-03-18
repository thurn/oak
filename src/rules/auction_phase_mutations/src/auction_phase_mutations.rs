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

use std::collections::{HashMap, HashSet};
use std::slice::ChunksExact;

use auction_phase_data::Contract;
use play_phase_data::{PlayPhaseData, Trick};
use primitives::{Card, HandIdentifier, PlayerName, Rank, Suit};
use rand::prelude::SliceRandom;
use rand::Rng;

pub fn new_game(rng: &mut impl Rng) -> PlayPhaseData {
    let mut cards = Vec::new();
    for suit in enum_iterator::all::<Suit>() {
        for rank in enum_iterator::all::<Rank>() {
            cards.push(Card::new(suit, rank))
        }
    }
    cards.shuffle(rng);

    let mut chunks = cards.chunks_exact(13);
    let mut hands = HashMap::new();
    hands.insert(HandIdentifier::North, build_hand(&mut chunks));
    hands.insert(HandIdentifier::East, build_hand(&mut chunks));
    hands.insert(HandIdentifier::South, build_hand(&mut chunks));
    hands.insert(HandIdentifier::West, build_hand(&mut chunks));

    PlayPhaseData {
        hands,
        current_trick: Trick::default(),
        completed_tricks: vec![],
        contract: Contract { declarer: PlayerName::User, trump: Some(Suit::Spades), bid: 8 },
    }
}

fn build_hand(chunks: &mut ChunksExact<Card>) -> HashSet<Card> {
    HashSet::from_iter(chunks.next().expect("Invalid deck size").iter().copied())
}
