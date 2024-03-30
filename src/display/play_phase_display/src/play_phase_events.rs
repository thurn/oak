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

use bevy::prelude::*;
use display_utils::object_display::{Displayable, ObjectDisplayPosition};
use play_phase_data::PlayPhaseData;
use primitives::{Card, HandIdentifier};

use crate::play_phase_spawn::CardComponent;

#[derive(Event)]
pub struct PlayPhaseUpdateEvent;

pub fn sync_state(
    mut commands: Commands,
    data: Res<PlayPhaseData>,
    mut updates: EventReader<PlayPhaseUpdateEvent>,
    cards: Query<(&CardComponent, Entity)>,
) {
    if !updates.is_empty() {
        updates.clear();
        for (card, entity) in cards.iter() {
            commands.entity(entity).insert(card_position(&data, card.data));
        }
    }
}

fn card_position(data: &PlayPhaseData, card: Card) -> Displayable {
    if let Some(position) = data.current_trick.cards.iter().position(|c| c.card == card) {
        return Displayable {
            position: ObjectDisplayPosition::InTrick(data.current_trick.cards[position].played_by),
            sorting_key: position,
        };
    }

    if let Some(position) = data
        .completed_tricks
        .iter()
        .flat_map(|completed| completed.trick.cards.iter())
        .position(|c| c.card == card)
    {
        return Displayable {
            position: ObjectDisplayPosition::CompletedTrick,
            sorting_key: position,
        };
    }

    for hand_id in enum_iterator::all::<HandIdentifier>() {
        let mut hand = data.hand(hand_id).collect::<Vec<_>>();
        hand.sort();
        if let Some(p) = hand.iter().position(|&c| c == card) {
            return Displayable {
                position: ObjectDisplayPosition::InHand(hand_id),
                sorting_key: p,
            };
        }
    }

    panic!("Card not found {card}")
}
