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

use play_phase_data::{PlayPhaseAction, PlayPhaseData, PlayedCard};
use primitives::{Card, HandIdentifier, PlayerName};

use crate::play_phase_flags;

pub fn handle_action(data: &mut PlayPhaseData, action: PlayPhaseAction) {
    match action {
        PlayPhaseAction::PlayCard(player, hand, card) => play_card(data, player, hand, card),
    }
}

/// Plays the indicated [Card] from the hand identified by [HandIdentifier] if
/// it is currently legal to do so.
fn play_card(data: &mut PlayPhaseData, _: PlayerName, hand: HandIdentifier, card: Card) {
    assert!(play_phase_flags::can_play_card(data, hand, card), "Cannot play {card}");
    data.hands.get_mut(&hand).unwrap().remove(&card);
    data.current_trick.cards.push(PlayedCard { played_by: hand, card });
}
