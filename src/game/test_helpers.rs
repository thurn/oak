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
    agents::constant::ConstantAgent,
    game::deck,
    model::{
        game::{Game, Trick},
        primitives::{Card, GamePhase, Position, Rank, Suit},
        state::State,
    },
};

/// Creates a game with the following starting configuration:
/// - Trump: None
/// - Phase: Bidding
/// - Lead: User
/// - Hands:
///   - User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
///   - Left:  ♦2 ♦3 ♦9 ♦10 ♦Q ♣4 ♣8 ♥5  ♥8 ♥K ♠3 ♠6 ♠A
///   - Dummy: ♦6 ♦7 ♦8 ♦K  ♣5 ♣K ♥4 ♥7  ♥J ♥Q ♠4 ♠5 ♠10
///   - Right: ♦4 ♦5 ♦J ♦A  ♣3 ♣7 ♣J ♣Q  ♥2 ♥3 ♠9 ♠J ♠Q
pub fn create_test_game() -> Game {
    deck::new_game(&mut Pcg64::seed_from_u64(17))
}

pub const USER_CARD_0: Card = Card {
    suit: Suit::Clubs,
    rank: Rank::Two,
};

/// Creates a [State] using [create_test_game] and [ConstantAgent], using the same
/// configuration:
/// - Trump: None
/// - Phase: Bidding
/// - Lead: User
/// - Hands:
///   - User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
///   - Left:  ♦2 ♦3 ♦9 ♦10 ♦Q ♣4 ♣8 ♥5  ♥8 ♥K ♠3 ♠6 ♠A
///   - Dummy: ♦6 ♦7 ♦8 ♦K  ♣5 ♣K ♥4 ♥7  ♥J ♥Q ♠4 ♠5 ♠10
///   - Right: ♦4 ♦5 ♦J ♦A  ♣3 ♣7 ♣J ♣Q  ♥2 ♥3 ♠9 ♠J ♠Q
pub fn create_test_state() -> State {
    State {
        game: create_test_game(),
        agent: Box::from(ConstantAgent {}),
    }
}

/// Creates a new game in the 'game over' state
pub fn create_empty_game() -> Game {
    Game {
        phase: GamePhase::Playing,
        trick: Trick::new(Position::User),
        trump: None,
        user_hand: vec![],
        dummy_hand: vec![],
        left_opponent_hand: vec![],
        right_opponet_hand: vec![],
    }
}
