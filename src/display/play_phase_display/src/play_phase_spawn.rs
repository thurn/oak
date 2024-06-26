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
use bevy_mod_picking::prelude::*;
use display_utils::anchored_transform::{AnchoredTransform, HorizontalAnchor, VerticalAnchor};
use display_utils::linear_display::{LinearDisplay, LinearDisplayDirection};
use display_utils::object_display::{ObjectDisplay, ObjectDisplayPosition};
use play_phase_data::{PlayPhaseAction, PlayPhaseData};
use play_phase_rules::{play_phase_actions, play_phase_flags};
use primitives::{Card, HandIdentifier, PlayerName};

use crate::play_phase_events::PlayPhaseUpdateEvent;

#[derive(Component)]
pub struct CardComponent {
    pub data: Card,
}

pub fn spawn(
    commands: &mut Commands,
    game: &PlayPhaseData,
    card_atlas: &CardAtlas,
    identifier: HandIdentifier,
) {
    let mut hand = game.hands.get(&identifier).unwrap().iter().collect::<Vec<_>>();
    hand.sort();
    let (horizontal, vertical) = match identifier {
        HandIdentifier::North => (HorizontalAnchor::Center, VerticalAnchor::Top),
        HandIdentifier::East => (HorizontalAnchor::Right, VerticalAnchor::Center),
        HandIdentifier::South => (HorizontalAnchor::Center, VerticalAnchor::Bottom),
        HandIdentifier::West => (HorizontalAnchor::Left, VerticalAnchor::Center),
    };
    let card_visible = match identifier {
        HandIdentifier::North | HandIdentifier::South => true,
        HandIdentifier::East | HandIdentifier::West => true,
    };
    let direction = match identifier {
        HandIdentifier::North | HandIdentifier::South => LinearDisplayDirection::Horizontal,
        HandIdentifier::East | HandIdentifier::West => LinearDisplayDirection::Vertical,
    };
    let sprite_anchor = match identifier {
        HandIdentifier::North => Anchor::TopCenter,
        HandIdentifier::East => Anchor::CenterRight,
        HandIdentifier::South => Anchor::BottomCenter,
        HandIdentifier::West => Anchor::CenterLeft,
    };

    commands
        .spawn((SpatialBundle::default(), AnchoredTransform { horizontal, vertical }))
        .with_children(|parent| {
            parent.spawn((
                ObjectDisplay { position: ObjectDisplayPosition::InHand(identifier) },
                SpatialBundle::default(),
                LinearDisplay { size: 225.0, direction },
            ));
        });

    commands
        .spawn((
            SpatialBundle::default(),
            AnchoredTransform {
                horizontal: HorizontalAnchor::Center,
                vertical: VerticalAnchor::Center,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ObjectDisplay { position: ObjectDisplayPosition::InTrick(identifier) },
                SpatialBundle::default(),
                LinearDisplay { size: 50.0, direction: LinearDisplayDirection::Vertical },
            ));
        });

    for &card in hand {
        let (texture, atlas) = card_atlas.get_card(card, card_visible);
        commands.spawn((
            CardComponent { data: card },
            SpriteSheetBundle {
                texture,
                atlas,
                sprite: Sprite { anchor: sprite_anchor, ..default() },
                ..default()
            },
            On::<Pointer<Click>>::run(
                move |mut data: ResMut<PlayPhaseData>,
                      mut updates: EventWriter<PlayPhaseUpdateEvent>| {
                    if play_phase_flags::can_play_card(&data, identifier, card) {
                        play_phase_actions::handle_action(
                            &mut data,
                            PlayPhaseAction::PlayCard(PlayerName::User, identifier, card),
                        );
                        updates.send(PlayPhaseUpdateEvent);
                    }
                },
            ),
        ));
    }
}
