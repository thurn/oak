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

use rand::{self, prelude::SliceRandom};
use std::collections::HashMap;
use std::fmt;
use std::iter;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew::prelude::*;
use yew::services;

fn olog(msg: &str) {
    services::ConsoleService::info(format!("Log: {:?}", msg).as_str());
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter, PartialOrd, Ord)]
enum Suit {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

impl Suit {
    fn is_red(&self) -> bool {
        match self {
            Suit::Clubs | Suit::Spades => false,
            Suit::Diamonds | Suit::Hearts => true,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
                Suit::Hearts => "♥",
                Suit::Spades => "♠",
            }
        )
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter, PartialOrd, Ord)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "10",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        )
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, PartialOrd, Ord)]
struct Card {
    suit: Suit,
    rank: Rank,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
pub enum Position {
    User,
    Left,
    Partner,
    Right,
}

impl Position {
    pub fn next(&self) -> Position {
        match self {
            Position::User => Position::Left,
            Position::Left => Position::Partner,
            Position::Partner => Position::Right,
            Position::Right => Position::User,
        }
    }
}

struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new(mut cards: Vec<Card>) -> Hand {
        cards.sort();
        Hand { cards }
    }

    pub fn remove(&mut self, index: usize) -> Card {
        self.cards.remove(index)
    }

    pub fn pick_lead(&self) -> Option<usize> {
        olog("Picking lead");
        self.cards
            .iter()
            .enumerate()
            .max_by_key(|(_, &value)| value)
            .map(|(idx, _)| idx)
    }

    pub fn pick_play(&self, suit: Suit) -> Option<usize> {
        if self.cards.iter().filter(|c| c.suit == suit).count() > 0 {
            olog("Following suit");
            self.cards
                .iter()
                .enumerate()
                .filter(|(_, c)| c.suit == suit)
                .max_by_key(|(_, &value)| value)
                .map(|(idx, _)| idx)
        } else {
            olog("Discarding");
            self.cards
                .iter()
                .enumerate()
                .min_by_key(|(_, &value)| value)
                .map(|(idx, _)| idx)
        }
    }
}

struct Trick {
    pub lead: Position,
    pub played: HashMap<Position, Card>,
}

impl Trick {
    pub fn winner(&self) -> Option<Position> {
        self.played
            .iter()
            .max_by_key(|(_, card)| *card)
            .map(|(position, _)| *position)
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumIter)]
enum GamePhase {
    Bidding,
    Playing,
}

struct Game {
    pub phase: GamePhase,
    pub hands: HashMap<Position, Hand>,
    pub trick: Trick,
}

impl Game {
    pub fn new() -> Game {
        let mut cards = Vec::new();
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push(Card { suit, rank })
            }
        }
        cards.shuffle(&mut rand::thread_rng());
        let mut chunks = cards.chunks_exact(13);
        let mut hands = HashMap::<Position, Hand>::new();
        hands.insert(Position::User, Hand::new(chunks.next().unwrap().to_vec()));
        hands.insert(Position::Left, Hand::new(chunks.next().unwrap().to_vec()));
        hands.insert(
            Position::Partner,
            Hand::new(chunks.next().unwrap().to_vec()),
        );
        hands.insert(Position::Right, Hand::new(chunks.next().unwrap().to_vec()));

        Game {
            phase: GamePhase::Bidding,
            hands,
            trick: Trick {
                lead: Position::User,
                played: HashMap::new(),
            },
        }
    }

    pub fn play_card(&mut self, position: Position, index: usize) {
        let card = self.hands.get_mut(&position).unwrap().remove(index);
        self.trick.played.insert(position, card);
    }

    pub fn play_position(&mut self, position: Position) {
        if self.trick.lead == position {
            let index = self.hands.get(&position).unwrap().pick_lead().unwrap();
            self.play_card(position, index);
        } else {
            let lead = self.trick.played.get(&self.trick.lead).unwrap().suit;
            self.play_card(
                position,
                self.hands.get(&position).unwrap().pick_play(lead).unwrap(),
            );
        }
    }
}

pub struct Model {
    link: ComponentLink<Self>,
    game: Game,
}

impl Model {
    fn render_card(&self, card: &Card, on_click: Option<Msg>, classes: Classes) -> Html {
        let mut class = classes!("card", "shown", "vertical");
        class.extend(classes);

        class.push(if card.suit.is_red() { "red" } else { "black" });
        if card.rank == Rank::Ten {
            class.push("ten");
        }

        let callback = on_click.map(|msg| self.link.callback(move |_| msg));

        html! {
            <div class=class draggable="true" onclick=callback>
                <div class="left">
                    <div class="rank">{card.rank}</div>
                    <div class="suit">{card.suit}</div>
                </div>
                <div class="center">
                    {card.suit}
                </div>
                <div class="right">
                    <div class="rank">{card.rank}</div>
                    <div class="suit">{card.suit}</div>
                </div>
            </div>
        }
    }

    fn render_card_wrapper(&self, (index, card): (usize, &Card), position: Position) -> Html {
        html! {
            <div class="card-wrapper">
                {self.render_card(card, Some(Msg::Play(position, index)), classes!())}
            </div>
        }
    }

    fn render_hidden_card(&self, position: Position) -> Html {
        let mut classes = classes!("card", "hidden");
        classes.push(match position {
            Position::User | Position::Partner => "vertical",
            Position::Left | Position::Right => "horizontal",
        });
        html! {
            <div class="card-wrapper">
                <div class=classes>
                    <div class="card-back" />
                </div>
            </div>
        }
    }

    fn render_play(&self, trick: &Trick, position: &Position) -> Option<Html> {
        let classes = match position {
            Position::User => "user-play",
            Position::Left => "left-play",
            Position::Partner => "partner-play",
            Position::Right => "right-play",
        };

        trick
            .played
            .get(position)
            .map(|c| self.render_card(c, None, classes!(classes)))
    }

    fn render_trick(&self, trick: &Trick) -> Html {
        if self.game.phase == GamePhase::Bidding {
            return html! {};
        }

        let callback = if trick.played.keys().len() == 4 {
            Some(self.link.callback(move |_| Msg::Continue))
        } else {
            None
        };

        html! {
            <div class="center trick" onclick=callback>
                {self.render_play(trick, &trick.lead).unwrap_or_default()}
                {self.render_play(trick, &trick.lead.next()).unwrap_or_default()}
                {self.render_play(trick, &trick.lead.next().next()).unwrap_or_default()}
                {self.render_play(trick, &trick.lead.next().next().next()).unwrap_or_default()}
            </div>
        }
    }

    fn render_partner_hand(&self) -> Html {
        if self.game.phase == GamePhase::Bidding {
            html! {
                <div class="hand user">
                    { for iter::repeat(self.render_hidden_card(Position::Partner))
                        .take(self.game.hands[&Position::Partner].cards.len())}
                </div>
            }
        } else {
            html! {
                <div class="hand user">
                    { for self.game.hands[&Position::Partner].cards.iter()
                        .enumerate().map(|e| self.render_card_wrapper(e, Position::Partner))
                    }
                </div>
            }
        }
    }

    fn render_bidding_buttons(&self) -> Html {
        if self.game.phase == GamePhase::Bidding {
            html! {
                <div class="center">
                    <div class="bidding">
                        <button class="bid red">{"♦?"}</button>
                        <button class="bid">{"♣?"}</button>
                        <button class="bid red">{"♥?"}</button>
                        <button class="bid">{"♠?"}</button>
                        <button class="bid">{"(?)"}</button>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Msg {
    Play(Position, usize),
    Continue,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            game: Game::new(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Play(position, index) => {
                olog("Playing");
                self.game.play_card(position, index);
                olog("AI move");
                if self.game.trick.played.keys().len() < 4 {
                    self.game.play_position(position.next());
                }
                true
            }
            Msg::Continue => {
                let winner = self.game.trick.winner().unwrap();
                self.game.trick.played.clear();
                self.game.trick.lead = winner;
                if winner == Position::Left || winner == Position::Right {
                    self.game.play_position(winner);
                }

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="content">
                {self.render_partner_hand()}
                <div class="main">
                    <div class="hand opponent">
                        { for iter::repeat(self.render_hidden_card(Position::Left))
                            .take(self.game.hands[&Position::Left].cards.len())}
                    </div>

                    {self.render_trick(&self.game.trick)}
                    {self.render_bidding_buttons()}

                    <div class="hand opponent">
                        { for iter::repeat(self.render_hidden_card(Position::Right))
                            .take(self.game.hands[&Position::Right].cards.len())}
                    </div>

                </div>
                <div class="hand user">
                    { for self.game.hands[&Position::User].cards.iter()
                        .enumerate().map(|e| self.render_card_wrapper(e, Position::User))
                    }
                </div>
            </div>
        }
    }
}
