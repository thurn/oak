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

use std::fmt::Display;

use yew::{prelude::*, virtual_dom::VNode};

use crate::{
    game::bidding_phase::HandScore,
    interface::{
        bid,
        main::{Action, Oak},
    },
    model::{
        bidding::{
            Auction,
            AuctionTurn,
            Bid,
            BidResponse,
            Bidder,
            HandBalance,
            HandRating,
            LengthOperator,
        },
        game::GameData,
        primitives::{Position, Suit},
    },
};

pub fn bid_button(link: &ComponentLink<Oak>, bid: Bid) -> Html {
    let mut classes = classes!("bid__bid-button");
    classes.push(match bid {
        Bid::Suit(s) if s.is_red() => "bid__bid-button--red",
        _ => "bid__bid-button--black",
    });
    let content = match bid {
        Bid::Query => "⊛".to_owned(),
        Bid::Suit(s) => format!("{}", s),
        Bid::Pass => "↷".to_owned(),
    };

    let onclick = link.callback(move |_| Action::Bid(bid));

    html! {
        <button class=classes onclick=onclick>
            {content}
        </button>
    }
}

pub fn bidding_controls(link: &ComponentLink<Oak>, game: &GameData) -> Html {
    html! {
        <div class="bid__bidding-controls">
            {bid_button(link, Bid::Query)}
            {bid_button(link, Bid::Suit(Suit::Diamonds))}
            {bid_button(link, Bid::Suit(Suit::Clubs))}
            {bid_button(link, Bid::Suit(Suit::Hearts))}
            {bid_button(link, Bid::Suit(Suit::Spades))}
            {bid_button(link, Bid::Pass)}
        </div>
    }
}

pub fn suit_span(suit: Suit) -> Html {
    let class = if suit.is_red() { "bid__suit-span--red" } else { "bid__suit-span--black" };
    html! {
        <span class=class>
            {suit}
        </span>
    }
}

pub fn bid_cell(turn: Option<&AuctionTurn>) -> Html {
    let formatted = match turn {
        Some(AuctionTurn { bid: Bid::Query, .. }) => html! {"⊛"},
        Some(AuctionTurn { bid: Bid::Suit(s), .. }) => suit_span(*s),
        Some(AuctionTurn { bid: Bid::Pass, .. }) => html! {"↷"},
        None => html! {},
    };

    html! {
        <div class="bid__history-cell bid__history-cell--bid">
            {formatted}
        </div>
    }
}

pub fn response_content(response: BidResponse) -> Html {
    let result = match response {
        BidResponse::Pass => html! {"Pass"},
        BidResponse::HandEvaluation(rating, _) => html! {format!("{} Hand", rating)},
        BidResponse::SuitLength(suit, count, op) => match op {
            LengthOperator::Lte => html! { <> {format!("≤ {}", count)} {suit_span(suit)} </> },
            LengthOperator::Gte => html! { <> {format!("≥ {}", count)} {suit_span(suit)} </> },
            LengthOperator::Equal => html! { <> {format!("= {}", count)} {suit_span(suit)} </> },
        },
        BidResponse::HandBalance(b) => match b {
            HandBalance::Balanced => html! {"Balanced"},
            HandBalance::Unbalanced => html! {"Unbalanced"},
        },
        BidResponse::LongestSuit(s) => html! { <> {"Longest:"} {suit_span(s)} </> },
        BidResponse::WeakestSuit(s) => html! { <> {"Weakest:"} {suit_span(s)} </> },
        BidResponse::RankCount(rank, count) => html! { format!("{} {}s", count, rank) },
    };

    html! {
        <div class="bid__response-content">
            {result}
        </div>
    }
}

pub fn response_cell(turn: Option<&AuctionTurn>) -> Html {
    let result = if let Some(t) = turn {
        html! {
            {for t.responses.iter().map(|r| response_content(*r))}
        }
    } else {
        html! {}
    };

    html! {
        <div class="bid__history-cell bid__history-cell--response">
            {result}
        </div>
    }
}

pub fn column_header(header: &str, is_response: bool) -> Html {
    let mut classes = classes!("bid__column-header", "bid__history-cell");
    classes.push(if is_response {
        "bid__history-cell--response"
    } else {
        "bid__history-cell--bid"
    });
    html! {
        <div class=classes>
            {header}
        </div>
    }
}

pub fn history_row(you: Html, r1: Html, them: Html, r2: Html) -> Html {
    html! {
        <div class="bid__history-row">
            {you}
            {r1}
            {them}
            {r2}
        </div>
    }
}

pub fn bid_history(auction: &Auction) -> Html {
    let user = if auction.first == Position::User { Bidder::First } else { Bidder::Second };

    let mut content = vec![history_row(
        column_header("You", false),
        column_header("Responses", true),
        column_header("Them", false),
        column_header("Responses", true),
    )];

    let mut i = 0;
    loop {
        content.push(history_row(
            bid_cell(auction.bids(user).get(i)),
            response_cell(auction.bids(user).get(i)),
            bid_cell(auction.bids(user.opposite()).get(i)),
            response_cell(auction.bids(user.opposite()).get(i)),
        ));

        i += 1;
        if i >= auction.bids(user).len() && i >= auction.bids(user.opposite()).len() {
            break;
        }
    }

    html! {
        <>
        <div class="bid__bid-history">
            {for content}
        </div>

        <div class="bid__bidding-for">
            <span class="bid__bidding-for__label">{"Bidding For:"}</span>
            <span>{format!("{} tricks", auction.bid_number + 1)}</span>
        </div>
        </>
    }
}

/// Renders the central square content for the bidding phase
pub fn render_bidding(link: &ComponentLink<Oak>, game: &GameData) -> Html {
    html! {
        <>
        {bidding_controls(link, game)}
        {bid_history(&game.auction)}
        </>
    }
}
