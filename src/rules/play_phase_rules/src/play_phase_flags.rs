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

use play_phase_data::PlayPhaseData;
use primitives::{Card, HandIdentifier};

use crate::play_phase_queries;

pub fn can_play_card(data: &PlayPhaseData, hand: HandIdentifier, card: Card) -> bool {
    if play_phase_queries::next_to_play(data) != hand {
        return false;
    }
    data.hands.get(&hand).unwrap().contains(&card)
}
