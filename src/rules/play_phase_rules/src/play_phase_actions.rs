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

use play_phase_data::{PlayPhaseData, PlayedCard};
use primitives::{Card, HandIdentifier};

use crate::play_phase_flags;

/// Plays the indicated [Card] from the hand identified by [HandIdentifier] if
/// it is currently legal to do so.
pub fn play_card_if_able(data: &mut PlayPhaseData, hand: HandIdentifier, card: Card) {
    if play_phase_flags::can_play_card(data, hand, card) {
        println!("Playing {card}");
        data.hands.get_mut(&hand).unwrap().remove(&card);
        data.current_trick.cards.push(PlayedCard { played_by: hand, card })
    }
}
