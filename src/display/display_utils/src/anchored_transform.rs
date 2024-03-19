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
use bevy::window::WindowResized;

/// Component for causing a Transform to be anchored to a screen-space position.
///
/// Entities using this component will be translated to the indicated vertical &
/// horizontal screen position whenever the window is resized.
#[derive(Component)]
pub struct AnchoredTransform {
    pub horizontal: HorizontalAnchor,
    pub vertical: VerticalAnchor,
}

pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

pub enum VerticalAnchor {
    Top,
    Center,
    Bottom,
}

pub fn on_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut query: Query<(&AnchoredTransform, &mut Transform)>,
) {
    for e in resize_reader.read() {
        for (anchored, mut transform) in query.iter_mut() {
            let (width, height) = (e.width, e.height);
            let x = match anchored.horizontal {
                HorizontalAnchor::Left => width / -2.0,
                HorizontalAnchor::Center => 0.0,
                HorizontalAnchor::Right => width / 2.0,
            };
            let y = match anchored.vertical {
                VerticalAnchor::Top => height / -2.0,
                VerticalAnchor::Center => 0.0,
                VerticalAnchor::Bottom => height / 2.0,
            };
            *transform = Transform::from_xyz(x, y, transform.translation.z);
        }
    }
}
