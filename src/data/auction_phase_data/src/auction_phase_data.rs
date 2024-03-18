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

use primitives::{PlayerName, Suit};

/// A bid for a number of tricks a player has committed to winning with a given
/// trump suit
#[derive(Debug, Clone)]
pub struct Contract {
    /// Player who bid for this contract value
    pub declarer: PlayerName,
    /// Trump suit for this contract, or None if the contract is for no trump.
    pub trump: Option<Suit>,
    /// Number of tricks the declarer has committed to winning
    pub bid: u32,
}
