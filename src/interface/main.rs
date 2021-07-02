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

//! Entry-point into our Yew application

use yew::prelude::*;

use crate::{
    agents::heuristic::HeuristicAgent,
    game::{bidding_phase, deck, play_phase},
    model::{
        game::Game,
        primitives::{Bid, CardId, Position},
        state::State,
    },
};

/// Represents possible actions taken by the user in the interface. In general
/// no error checking is performed for actions -- it is assumed that the
/// interface will only allow valid actions.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Action {
    /// Play the card with the given [CardId]
    Play(CardId),

    /// Request to clear the current trick and start a new one
    Continue,

    /// Place a bid during the bidding phase
    Bid(Bid),
}

pub struct Oak {
    state: State,
    pub link: ComponentLink<Self>,
}

impl Component for Oak {
    type Message = Action;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            state: State {
                game: deck::new_game(&mut rand::thread_rng()),
                agent: Box::from(HeuristicAgent {}),
            },
            link,
        }
    }

    fn update(&mut self, action: Action) -> ShouldRender {
        match action {
            Action::Play(card_id) => play_phase::resolve_card_play_action(&mut self.state, card_id),
            Action::Continue => play_phase::resolve_continue_action(&mut self.state),
            Action::Bid(bid) => bidding_phase::resolve_bid_action(&mut self.state, bid),
        };
        true
    }

    fn change(&mut self, _: ()) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        super::game::render_game(&self.link, &self.state.game)
    }
}
