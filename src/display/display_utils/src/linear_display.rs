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

/// Controls whether a [LinearDisplay] shows its contents in a horizontal row or
/// vertical column
pub enum LinearDisplayDirection {
    Horizontal,
    Vertical,
}

/// Component which translates its children to evenly distribute their X
/// positions within a given width.
///
/// Children will be positioned at x coordinates between -width/2 and width/2.
#[derive(Component)]
pub struct LinearDisplay {
    /// Width (for horizontal) or height (for vertical) for the display to
    /// occupy, in logical pixels
    pub size: f32,
    /// Controls whether the display shows its contents in a horizontal row or
    /// vertical column
    pub direction: LinearDisplayDirection,
}

pub fn update(query: Query<(&LinearDisplay, &Children)>, mut transforms: Query<&mut Transform>) {
    for (display, children_iterator) in query.iter() {
        let children = children_iterator.iter().collect::<Vec<_>>();
        let count = children.len();
        match count {
            0 => {}
            1 => {
                let mut transform = transforms.get_mut(*children[0]).unwrap();
                match display.direction {
                    LinearDisplayDirection::Horizontal => {
                        transform.translation.x = 0.0;
                    }
                    LinearDisplayDirection::Vertical => {
                        transform.translation.y = 0.0;
                    }
                }
                transform.translation.z = 0.0;
            }
            _ => {
                let increment = display.size / (count - 1) as f32;
                for (i, child) in children.into_iter().enumerate() {
                    let mut transform = transforms.get_mut(*child).unwrap();
                    let offset = (-display.size / 2.0) + (i as f32 * increment);
                    match display.direction {
                        LinearDisplayDirection::Horizontal => {
                            transform.translation.x = offset;
                            transform.translation.z = i as f32;
                        }
                        LinearDisplayDirection::Vertical => {
                            transform.translation.y = offset;
                            transform.translation.z = (count - i) as f32;
                        }
                    }
                }
            }
        }
    }
}
