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
use primitives::HandIdentifier;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub enum ObjectDisplayPosition {
    InHand(HandIdentifier),
    InTrick(HandIdentifier),
    CompletedTrick,
}

/// Identifies a world space position to which a game object can be moved.
#[derive(Component, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Displayable {
    /// Anchor position within the world at which to place this object
    pub position: ObjectDisplayPosition,
    /// Determines Z index for displayed items.
    ///
    /// Objects with higher sorting_key numbers will be rendered on top of lower
    /// numbers.
    pub sorting_key: usize,
}

/// Marks entities which perform layout on their children in world space.
/// Children must have a [Displayable] component attached to participate.
#[derive(Component)]
pub struct ObjectDisplay {
    /// Uniquely identifies this ObjectDisplay and indicates its position.
    pub position: ObjectDisplayPosition,
}

pub fn update(
    mut commands: Commands,
    parent_query: Query<(Entity, &ObjectDisplay)>,
    children_query: Query<(Entity, &Displayable)>,
) {
    let mut displayables = children_query.iter().collect::<Vec<_>>();
    displayables.sort_by_key(|(_, d)| *d);
    for (parent, _) in &parent_query {
        commands.entity(parent).clear_children();
    }

    for (child, displayable) in &displayables {
        if let Some((parent, _)) =
            parent_query.iter().find(|(_, display)| display.position == displayable.position)
        {
            commands.entity(parent).add_child(*child);
        }
    }
}
