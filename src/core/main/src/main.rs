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

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::WindowResized;

use assets::CardAtlas;
use display_utils::anchored_transform::{AnchoredTransform, HorizontalAnchor, VerticalAnchor};
use display_utils::linear_row::LinearRow;
use display_utils::plugin::DisplayUtilsPlugin;
use play_phase_data::PlayPhaseData;
use primitives::HandIdentifier;

const MARKER_SIDE_LENGTH: f32 = 25.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DisplayUtilsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, on_resize_system)
        .run();
}

#[derive(Debug)]
enum MarkerPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Component)]
struct MarkerComponent {
    pub position: MarkerPosition,
}

#[derive(Bundle)]
struct MarkerBundle {
    marker: MarkerComponent,
    sprite_bundle: SpriteBundle,
}

//takes a transform specifying its position, and color of the sprite / marker
impl MarkerBundle {
    fn new(position: MarkerPosition, transform: Transform, color: Color) -> Self {
        Self {
            marker: MarkerComponent { position },
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(MARKER_SIDE_LENGTH, MARKER_SIDE_LENGTH)),
                    anchor: bevy::sprite::Anchor::TopLeft,
                    ..default()
                },
                transform,
                ..default()
            },
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());
    let game = auction_phase_mutations::new_game(&mut rand::thread_rng());
    let card_atlas = CardAtlas::new(asset_server, texture_atlas_layouts);
    spawn_hand(
        &mut commands,
        &game,
        &card_atlas,
        HandIdentifier::North,
        Anchor::TopCenter,
        VerticalAnchor::Top,
    );
    spawn_hand(
        &mut commands,
        &game,
        &card_atlas,
        HandIdentifier::South,
        Anchor::BottomCenter,
        VerticalAnchor::Bottom,
    );
}

fn spawn_hand(
    commands: &mut Commands,
    game: &PlayPhaseData,
    card_atlas: &CardAtlas,
    identifier: HandIdentifier,
    anchor: Anchor,
    vertical_anchor: VerticalAnchor,
) {
    let mut hand = game.hands.get(&identifier).unwrap().iter().collect::<Vec<_>>();
    hand.sort();

    let mut h = commands.spawn((
        SpatialBundle::default(),
        AnchoredTransform { horizontal: HorizontalAnchor::Center, vertical: vertical_anchor },
    ));
    h.with_children(|parent| {
        parent.spawn((SpatialBundle::default(), LinearRow { width: 225.0 })).with_children(
            |parent| {
                for card in hand {
                    let (texture, atlas) = card_atlas.get_card(*card);
                    parent.spawn(SpriteSheetBundle {
                        texture,
                        atlas,
                        sprite: Sprite { anchor, ..default() },
                        ..default()
                    });
                }
            },
        );
    });
}

fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut markers: Query<(&MarkerComponent, &mut Transform)>,
) {
    for e in resize_reader.read() {
        for (marker, mut transform) in markers.iter_mut() {
            let (width, height) = (e.width, e.height);
            match marker.position {
                MarkerPosition::TopLeft => {
                    *transform = Transform::from_xyz(width / -2.0, height / 2.0, 0.0);
                }
                MarkerPosition::TopRight => {
                    *transform =
                        Transform::from_xyz(width / 2.0 - MARKER_SIDE_LENGTH, height / 2.0, 0.0);
                }
                MarkerPosition::BottomLeft => {
                    *transform =
                        Transform::from_xyz(width / -2.0, height / -2.0 + MARKER_SIDE_LENGTH, 0.0);
                }
                MarkerPosition::BottomRight => {
                    *transform = Transform::from_xyz(
                        width / 2.0 - MARKER_SIDE_LENGTH,
                        height / -2.0 + MARKER_SIDE_LENGTH,
                        0.0,
                    );
                }
            }
        }
    }
}
