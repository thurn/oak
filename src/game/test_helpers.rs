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
    agents::{agent, constant::ConstantAgent},
    game::deck,
    model::{
        game::{Contract, GameData, GamePhase, PlayPhaseData, Trick},
        primitives::{Card, Position, Rank, Suit},
        state::State,
    },
};

/// Creates a [PlayPhaseData] with the following starting configuration:
/// - Contract: No Trump @ 7 tricks, User is declarer
/// - Lead: User
/// - Hands:
///   - User:  ♣2 ♣6 ♣9 ♣10 ♣A ♥6 ♥9 ♥10 ♥A ♠2 ♠7 ♠8 ♠K
///   - Left:  ♦2 ♦3 ♦9 ♦10 ♦Q ♣4 ♣8 ♥5  ♥8 ♥K ♠3 ♠6 ♠A
///   - Dummy: ♦6 ♦7 ♦8 ♦K  ♣5 ♣K ♥4 ♥7  ♥J ♥Q ♠4 ♠5 ♠10
///   - Right: ♦4 ♦5 ♦J ♦A  ♣3 ♣7 ♣J ♣Q  ♥2 ♥3 ♠9 ♠J ♠Q
pub fn create_test_play_phase() -> PlayPhaseData {
    PlayPhaseData {
        game: deck::new_game(&mut Pcg64::seed_from_u64(17), Position::User, Position::Right),
        trick: Trick::new(Position::User),
        contract: Contract { trump: None, tricks: 7, declarer: Position::User },
    }
}

/// Creates a [GameData] with the following configuration:
/// - First Bidder: User
/// - Second Bidder: Right
/// - Hands: as in [create_test_play_phase]
pub fn create_test_bid_phase() -> GameData {
    deck::new_game(&mut Pcg64::seed_from_u64(17), Position::User, Position::Right)
}

pub const USER_CARD_0: Card = Card { suit: Suit::Clubs, rank: Rank::Two };

/// Creates a [State] using [create_test_game] and [ConstantAgent], using the
/// same configuration.
pub fn create_test_state() -> State {
    let data = create_test_play_phase();
    let agent = create_test_agent();
    State { phase: GamePhase::Playing(data), agent }
}

pub fn create_test_agent() -> Box<dyn agent::Agent> {
    Box::from(ConstantAgent {})
}

/// Creates a new game in the 'game over' state
pub fn create_empty_game() -> PlayPhaseData {
    let mut result = create_test_play_phase();
    result.game.hands.user_hand.clear();
    result.game.hands.left_opponent_hand.clear();
    result.game.hands.dummy_hand.clear();
    result.game.hands.right_opponet_hand.clear();
    result
}
