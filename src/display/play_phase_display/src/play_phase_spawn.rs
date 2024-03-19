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

use assets::CardAtlas;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use display_utils::anchored_transform::{AnchoredTransform, HorizontalAnchor, VerticalAnchor};
use display_utils::linear_display::{LinearDisplay, LinearDisplayDirection};
use play_phase_data::PlayPhaseData;
use primitives::HandIdentifier;

pub fn spawn_hand(
    commands: &mut Commands,
    game: &PlayPhaseData,
    card_atlas: &CardAtlas,
    identifier: HandIdentifier,
) {
    let mut hand = game.hands.get(&identifier).unwrap().iter().collect::<Vec<_>>();
    hand.sort();
    let (horizontal, vertical) = match identifier {
        HandIdentifier::North => (HorizontalAnchor::Center, VerticalAnchor::Top),
        HandIdentifier::East => (HorizontalAnchor::Left, VerticalAnchor::Center),
        HandIdentifier::South => (HorizontalAnchor::Center, VerticalAnchor::Bottom),
        HandIdentifier::West => (HorizontalAnchor::Right, VerticalAnchor::Center),
    };
    let card_visible = match identifier {
        HandIdentifier::North | HandIdentifier::South => true,
        HandIdentifier::East | HandIdentifier::West => false,
    };
    let direction = match identifier {
        HandIdentifier::North | HandIdentifier::South => LinearDisplayDirection::Horizontal,
        HandIdentifier::East | HandIdentifier::West => LinearDisplayDirection::Vertical,
    };
    let sprite_anchor = match identifier {
        HandIdentifier::North => Anchor::TopCenter,
        HandIdentifier::East => Anchor::CenterLeft,
        HandIdentifier::South => Anchor::BottomCenter,
        HandIdentifier::West => Anchor::CenterRight,
    };

    let mut h =
        commands.spawn((SpatialBundle::default(), AnchoredTransform { horizontal, vertical }));
    h.with_children(|parent| {
        parent
            .spawn((SpatialBundle::default(), LinearDisplay { size: 225.0, direction }))
            .with_children(|parent| {
                for card in hand {
                    let (texture, atlas) = card_atlas.get_card(*card, card_visible);
                    parent.spawn(SpriteSheetBundle {
                        texture,
                        atlas,
                        sprite: Sprite { anchor: sprite_anchor, ..default() },
                        ..default()
                    });
                }
            });
    });
}
