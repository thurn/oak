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

/// Component which translates its children to evenly distribute their X
/// positions within a given width.
///
/// Children will be positioned at x coordinates between -width/2 and width/2.
#[derive(Component)]
pub struct LinearRow {
    /// Width for the row to occupy, in logical pixels
    pub width: f32,
}

pub fn process_rows(rows: Query<(&LinearRow, &Children)>, mut transforms: Query<&mut Transform>) {
    for (row, children_iterator) in rows.iter() {
        let children = children_iterator.iter().collect::<Vec<_>>();
        let count = children.len();
        match count {
            0 => {}
            1 => {
                transforms.get_mut(*children[0]).unwrap().translation.x = 0.0;
            }
            2 => {
                transforms.get_mut(*children[0]).unwrap().translation.x = -row.width / 2.0;
                transforms.get_mut(*children[1]).unwrap().translation.x = row.width / 2.0;
            }
            _ => {
                let increment = row.width / (count - 1) as f32;
                for (i, child) in children.into_iter().enumerate() {
                    let mut transform = transforms.get_mut(*child).unwrap();
                    transform.translation.x = (-row.width / 2.0) + (i as f32 * increment);
                    transform.translation.z = i as f32;
                }
            }
        }
    }
}
