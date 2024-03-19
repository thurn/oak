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
use bevy::window::WindowResized;
use display_utils::linear_row::LinearRow;
use display_utils::plugin::DisplayUtilsPlugin;
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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());
    let game = auction_phase_mutations::new_game(&mut rand::thread_rng());
    let hand = game.hands.get(&HandIdentifier::South).unwrap();
    let texture = asset_server.load("cards/sprite.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(75.0, 112.5), 14, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands
        .spawn((
            TransformBundle::default(),
            VisibilityBundle::default(),
            LinearRow { width: 400.0 },
        ))
        .with_children(|parent| {
            for (i, _) in hand.iter().enumerate() {
                println!("Spawning {i}");
                parent.spawn(SpriteSheetBundle {
                    texture: texture.clone(),
                    atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: i },
                    ..default()
                });
            }
        });
}

// fn setup(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
// ) {
//     let texture = asset_server.load("cards/sprite.png");
//     let layout = TextureAtlasLayout::from_grid(Vec2::new(75.0, 112.5), 14, 4,
// None, None);     let texture_atlas_layout =
// texture_atlas_layouts.add(layout);     commands.
// spawn(Camera2dBundle::default());     commands.spawn(SpriteSheetBundle {
//         texture: texture.clone(),
//         atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 0
// },         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
//             .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.0, 0.0,
// f32::to_radians(20.0))),         ..default()
//     });
//     commands.spawn(SpriteSheetBundle {
//         texture: texture.clone(),
//         atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 1
// },         transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0))
//             .with_rotation(Quat::from_euler(EulerRot::YXZ,
// f32::to_radians(20.0), 0.0, 0.0)),         ..default()
//     });
//     commands.spawn(SpriteSheetBundle {
//         texture: texture.clone(),
//         atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 2
// },         transform: Transform::from_translation(Vec3::new(200.0, 0.0, 0.0))
//             .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.0,
// f32::to_radians(20.0), 0.0)),         ..default()
//     });
//
//     let width = 1280.0;
//     let height = 720.0;
//
//     //TOP LEFT
//     commands.spawn(MarkerBundle::new(
//         MarkerPosition::TopLeft,
//         Transform::from_xyz(width / -2.0, height / 2.0, 0.0),
//         Color::GREEN,
//     ));
//
//     //BOTTOM LEFT
//     commands.spawn(MarkerBundle::new(
//         MarkerPosition::BottomLeft,
//         Transform::from_xyz(width / -2.0, height / -2.0 + MARKER_SIDE_LENGTH,
// 0.0),         Color::RED,
//     ));
//
//     //TOP RIGHT
//     commands.spawn(MarkerBundle::new(
//         MarkerPosition::TopRight,
//         Transform::from_xyz(width / 2.0 - MARKER_SIDE_LENGTH, height / 2.0,
// 0.0),         Color::ORANGE,
//     ));
//
//     //BOTTOM RIGHT
//     commands.spawn(MarkerBundle::new(
//         MarkerPosition::BottomRight,
//         Transform::from_xyz(
//             width / 2.0 - MARKER_SIDE_LENGTH,
//             height / -2.0 + MARKER_SIDE_LENGTH,
//             0.0,
//         ),
//         Color::PURPLE,
//     ));
// }

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
