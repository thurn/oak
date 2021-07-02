// Copyright © 2021-present Derek Thurn

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Defines the trait implemented by all AI agents

use std::fmt;

use crate::model::{
    game::Game,
    primitives::{CardId, Position},
};

pub trait Agent: fmt::Debug {
    /// Invoked during the Play phase when it's the agent's turn to play a
    /// card, either to lead a new trick or to follow an existing one. Should
    /// return the index of a card in hand to play.
    fn select_play(&self, game: &Game, position: Position) -> usize;
}
