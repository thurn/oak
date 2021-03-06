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

use anyhow::anyhow;
use yew::prelude::*;

use crate::{
    agents::heuristic::HeuristicAgent,
    game::{bidding_phase, deck, play_phase},
    interface::game,
    model::{
        bidding::Bid,
        game::GamePhase,
        primitives::{CardId, Position},
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
                phase: GamePhase::Auction(deck::new_game(
                    &mut rand::thread_rng(),
                    Position::User,
                    Position::Left,
                )),
                agent: Box::from(HeuristicAgent {}),
            },
            link,
        }
    }

    fn update(&mut self, action: Action) -> ShouldRender {
        let result = match action {
            Action::Play(card_id) => match self.state.phase {
                GamePhase::Playing(ref mut data) => {
                    play_phase::resolve_card_play_action(data, &*self.state.agent, card_id)
                }
                _ => Err(anyhow!("Can only play cards during the Play phase")),
            },
            Action::Continue => match self.state.phase {
                GamePhase::Playing(ref mut data) => {
                    play_phase::resolve_continue_action(data, &*self.state.agent)
                }
                _ => Err(anyhow!("Can only continue during the Play phase")),
            },
            Action::Bid(bid) => {
                bidding_phase::resolve_bid_action(&mut self.state.phase, &*self.state.agent, bid)
            }
        };

        if let Err(e) = result {
            panic!("Error: {:?}", e);
        }

        true
    }

    fn change(&mut self, _: ()) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match &self.state.phase {
            GamePhase::Auction(game) => game::render_game(&self.link, game, None),
            GamePhase::Playing(data) => game::render_game(&self.link, &data.game, Some(data)),
            _ => html! {},
        }
    }
}
