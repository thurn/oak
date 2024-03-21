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

use assets::CardAtlas;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use display_utils::plugin::DisplayUtilsPlugin;
use play_phase_display::play_phase_spawn;
use primitives::HandIdentifier;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(DisplayUtilsPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());
    let game = auction_phase_mutations::new_game(&mut rand::thread_rng());
    let card_atlas = CardAtlas::new(asset_server, texture_atlas_layouts);

    play_phase_spawn::spawn_hand(&mut commands, &game, &card_atlas, HandIdentifier::North);
    play_phase_spawn::spawn_hand(&mut commands, &game, &card_atlas, HandIdentifier::East);
    play_phase_spawn::spawn_hand(&mut commands, &game, &card_atlas, HandIdentifier::South);
    play_phase_spawn::spawn_hand(&mut commands, &game, &card_atlas, HandIdentifier::West);
    commands.insert_resource(game);
}
