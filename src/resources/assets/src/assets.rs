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
use primitives::{Card, Rank, Suit};

pub fn load_card(asset_server: Res<AssetServer>) -> Handle<Image> {
    asset_server.load("cards/clubKing.png")
}

pub struct CardAtlas {
    atlas: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl CardAtlas {
    pub fn new(
        asset_server: Res<AssetServer>,
        mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) -> Self {
        Self {
            atlas: asset_server.load("cards/sprite.png"),
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                Vec2::new(75.0, 112.5),
                14,
                4,
                None,
                None,
            )),
        }
    }

    pub fn get_card(&self, card: Card) -> (Handle<Image>, TextureAtlas) {
        let suit_offset = match card.suit {
            Suit::Clubs => 0,
            Suit::Diamonds => 14,
            Suit::Hearts => 28,
            Suit::Spades => 42,
        };
        let rank_offset = match card.rank {
            Rank::Two => 1,
            Rank::Three => 2,
            Rank::Four => 3,
            Rank::Five => 4,
            Rank::Six => 5,
            Rank::Seven => 6,
            Rank::Eight => 7,
            Rank::Nine => 8,
            Rank::Ten => 9,
            Rank::Jack => 10,
            Rank::Queen => 11,
            Rank::King => 12,
            Rank::Ace => 0,
        };

        (
            self.atlas.clone(),
            TextureAtlas { layout: self.layout.clone(), index: suit_offset + rank_offset },
        )
    }
}
