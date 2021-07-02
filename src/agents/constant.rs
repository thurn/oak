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

//! The constant agent always selects the first legal play available to it

use crate::{
    agents::agent::Agent,
    game::play_phase,
    model::{game::PlayPhaseData, primitives::Position},
};

#[derive(Debug)]
pub struct ConstantAgent;

impl Agent for ConstantAgent {
    fn select_play(&self, data: &PlayPhaseData, position: Position) -> usize {
        play_phase::legal_plays(data, position).map(|(i, _)| i).next().expect("No legal plays")
    }
}
