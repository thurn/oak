// Copyright © Vow 2024-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::AssetSource;
use bevy::prelude::*;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("other://cards/clubKing.png"),
        ..default()
    });
}

fn main() {
    App::new()
        .register_asset_source(
            "other",
            AssetSource::build().with_reader(|| Box::new(FileAssetReader::new("../../../assets"))),
        )
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
