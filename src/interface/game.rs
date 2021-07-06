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

//! Reusable UI components for rendering the state of a game

use std::{collections::HashSet, iter};

use yew::prelude::*;

use crate::{
    game::play_phase,
    interface::{
        bid,
        main::{Action, Oak},
    },
    model::{
        game::{GameData, GamePhase, PlayPhaseData, Trick},
        primitives::{Card, CardId, Position, Rank},
    },
};

type OnClick = Option<Callback<MouseEvent>>;

/// Primary container for all other UI
pub fn main_frame(content: Html) -> Html {
    html! {
        <div class="game__main-frame">
            {content}
        </div>
    }
}

/// Renders the full-width content in the middle of the [main_frame], typically
/// contains all UI *except* the user & partner hands.
pub fn middle_panel(content: Html) -> Html {
    html! {
        <div class="game__middle-panel">
            {content}
        </div>
    }
}

/// Renders the content in the center of the table, displaying the UI which
/// appears in between the four hands
pub fn central_square(content: Html, on_click: OnClick) -> Html {
    html! {
        <div class="game__central-square" onclick=on_click>
            {content}
        </div>
    }
}

/// Renders the primary horizontal hand displays (user & partner), but not
/// the opponent hands
pub fn hand_row(
    link: &ComponentLink<Oak>,
    game: &GameData,
    play_phase: Option<&PlayPhaseData>,
    position: Position,
    hidden: bool,
) -> Html {
    let legal_plays = match play_phase {
        Some(data) => play_phase::legal_plays(data, position)
            .map(|(index, _)| index)
            .collect::<HashSet<usize>>(),
        _ => HashSet::new(),
    };

    let content = game.hand(position).iter().enumerate().map(|(index, card)| {
        let callback = legal_plays
            .contains(&index)
            .then(|| link.callback(move |_| Action::Play(CardId::new(position, index))));
        card_in_hand(
            *card,
            hidden,
            CardOrientation::Vertical,
            callback,
            game.debug.show_hidden_cards,
        )
    });

    html! {
        <div class="game__hand-row">
            {for content}
        </div>
    }
}

/// Renders a column showing opponents' hands
pub fn opponent_hand_column(cards: &[Card], show_hidden: bool) -> Html {
    html! {
        <div class="game__opponent-hand-column">
        {
            for cards
                .iter()
                .map(|card|
                    card_in_hand(*card, true, CardOrientation::Horizontal, None, show_hidden))
        }
        </div>
    }
}

/// Renders a single vertical card contained in the [hand_row] or
/// [opponent_hand_column]
pub fn card_in_hand(
    card: Card,
    hidden: bool,
    orientation: CardOrientation,
    on_click: OnClick,
    show_hidden: bool,
) -> Html {
    let content = if hidden {
        hidden_card(card, orientation, show_hidden)
    } else {
        visible_card(card, on_click)
    };

    html! {
        <div class="game__card-in-hand">
            {content}
        </div>
    }
}

/// Primary function for rendering a face-up card, either in hand or in a trick
pub fn visible_card(card: Card, on_click: OnClick) -> Html {
    let mut classes = classes!("game__visible-card");
    classes.push(if card.suit.is_red() {
        "game__visible-card--red"
    } else {
        "game__visible-card--black"
    });
    classes.push(if on_click.is_some() {
        "game__visible-card--playable"
    } else {
        "game__visible-card--unplayable"
    });

    if card.rank == Rank::Ten {
        classes.push("game__visible-card--ten");
    }

    html! {
        <div class=classes draggable="true" onclick=on_click>
            <div class="game__visible-card__left">
                <div class="game__visible-card__rank">{card.rank}</div>
                <div class="game__visible-card__suit">{card.suit}</div>
            </div>
            <div class="game__visible-card__center">
                {card.suit}
            </div>
            <div class="game__visible-card__right">
                <div class="game__visible-card__rank">{card.rank}</div>
                <div class="game__visible-card__suit">{card.suit}</div>
            </div>
        </div>
    }
}

/// Direction of cards
pub enum CardOrientation {
    Vertical,
    Horizontal,
}

/// Renders a face-down card in a given [CardOrientation]
pub fn hidden_card(card: Card, orientation: CardOrientation, show_hidden: bool) -> Html {
    let mut classes = classes!("game__hidden-card");
    classes.push(match orientation {
        CardOrientation::Vertical => "game__hidden-card--vertical",
        CardOrientation::Horizontal => "game__hidden-card--horizontal",
    });
    let content = if show_hidden {
        html! {
            <div class="game__debug-card-info">
                {card.rank} {card.suit}
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class=classes>
            <div class="game__hidden-card__card-back" />
            {content}
        </div>
    }
}

pub fn current_trick(trick: &Trick) -> Html {
    let content = trick.cards().map(|(position, card)| {
        let class = match position {
            Position::User => "game__current-trick__user",
            Position::Left => "game__current-trick__left",
            Position::Dummy => "game__current-trick__dummy",
            Position::Right => "game__current-trick__right",
        };
        html! {
            <div class=class>
                {visible_card(card, None)}
            </div>
        }
    });

    html! {
        <div class="game__current-trick">
            {for content}
        </div>
    }
}

/// Renders the full content for a Game
pub fn render_game(
    link: &ComponentLink<Oak>,
    game: &GameData,
    play_phase: Option<&PlayPhaseData>,
) -> Html {
    let (center_content, on_click, hide_dummy) = match play_phase {
        None => (bid::render_bidding(link, game), None, true),
        Some(play_data) => (
            current_trick(&play_data.trick),
            play_data.trick.is_completed().then(|| link.callback(|_| Action::Continue)),
            false,
        ),
    };

    main_frame(html! {
        <>
        {hand_row(link, game, play_phase, Position::Dummy, hide_dummy)}
        {middle_panel(html! {
            <>
                {opponent_hand_column(game.hand(Position::Left), game.debug.show_hidden_cards)}
                {central_square(center_content, on_click)}
                {opponent_hand_column(game.hand(Position::Right), game.debug.show_hidden_cards)}
            </>
        })}
        {hand_row(link, game, play_phase, Position::User, false)}
        </>
    })
}
