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

//! Utilities for writing unit tests

#![cfg(test)]

use rand::SeedableRng;
use rand_pcg::Pcg64;

use crate::{
    game::deck,
    model::{
        game::Game,
        primitives::{Card, Rank, Suit},
    },
};

/// Creates a game with the following starting configuration:
/// - Trump: None
/// - Phase: Bidding
/// - Lead: User
/// - Hands:
///   - User:  10♣ 8♠ 6♥ A♥ 9♣ 7♠ 10♥ 2♠ 9♥ 6♣ K♠ A♣ 2♣
///   - Left:  10♦ 9♦ 4♣ 3♦ 6♠ 2♦ Q♦ 8♣ K♥ 5♥ 8♥ A♠ 3♠
///   - Dummy: 4♥ 10♠ K♦ 8♦ 5♣ Q♥ 7♦ 7♥ J♥ 6♦ K♣ 4♠ 5♠
///   - Right: 3♣ 2♥ 5♦ J♦ 7♣ A♦ 4♦ J♣ 3♥ Q♣ Q♠ 9♠ J♠
pub fn create_test_game() -> Game {
    deck::new_game(&mut Pcg64::seed_from_u64(17))
}

pub const USER_CARD_0: Card = Card {
    suit: Suit::Clubs,
    rank: Rank::Ten,
};
