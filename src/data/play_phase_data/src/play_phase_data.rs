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

use std::collections::{HashMap, HashSet};

use auction_phase_data::Contract;
use primitives::{Card, HandIdentifier};

#[derive(Debug, Clone)]
pub struct PlayPhaseData {
    pub hands: HashMap<HandIdentifier, HashSet<Card>>,
    pub current_trick: Trick,
    pub completed_tricks: Vec<CompletedTrick>,
    pub contract: Contract,
}

#[derive(Debug, Clone)]
pub struct CompletedTrick {
    /// Cards which were played in this trick.
    pub trick: Trick,
    /// Player who won this trick
    pub winner: HandIdentifier,
}

#[derive(Debug, Clone, Default)]
pub struct Trick {
    /// Cards played in this trick, in sequence
    pub cards: Vec<PlayedCard>,
}

/// Represents a card played to a trick
#[derive(Debug, Clone)]
pub struct PlayedCard {
    /// Player who played this card
    pub played_by: HandIdentifier,
    /// Card which was played
    pub card: Card,
}
