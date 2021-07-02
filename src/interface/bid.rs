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

//! UI components for the bidding phase

use yew::prelude::*;

use crate::{interface::bid, model::game::Game};

use super::main::Oak;

pub fn bidding_controls(link: &ComponentLink<Oak>, game: &Game) -> Html {
    let red = "bid__bidding-controls__bid-button bid__bidding-controls__bid-button--red";
    let black = "bid__bidding-controls__bid-button bid__bidding-controls__bid-button--black";

    html! {
        <div class="bid__bidding-controls">
            <button class=red>{"♦?"}</button>
            <button class=black>{"♣?"}</button>
            <button class=red>{"♥?"}</button>
            <button class=black>{"♠?"}</button>
            <button class=black>{"(?)"}</button>
        </div>
    }
}

/// Renders the central square content for the bidding phase
pub fn render_bidding(link: &ComponentLink<Oak>, game: &Game) -> Html {
    html! {
        {bidding_controls(link, game)}
    }
}
