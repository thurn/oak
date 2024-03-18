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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("cards/sprite.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(75.0, 112.5), 14, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteSheetBundle {
        texture: texture.clone(),
        atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 0 },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
    commands.spawn(SpriteSheetBundle {
        texture: texture.clone(),
        atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 1 },
        transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        ..default()
    });
    commands.spawn(SpriteSheetBundle {
        texture: texture.clone(),
        atlas: TextureAtlas { layout: texture_atlas_layout.clone(), index: 2 },
        transform: Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
        ..default()
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .run();
}
