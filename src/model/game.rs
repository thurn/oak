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

//! Types related to a single game

use std::iter;

use strum::IntoEnumIterator;

use crate::model::primitives::{Card, Position, Suit};

/// The current trick being played
#[derive(Debug, PartialEq, Eq)]
pub struct Trick {
    pub lead: Position,
    pub user_play: Option<Card>,
    pub dummy_play: Option<Card>,
    pub left_play: Option<Card>,
    pub right_play: Option<Card>,
}

impl Trick {
    pub fn new(lead: Position) -> Self {
        Self { lead, user_play: None, dummy_play: None, left_play: None, right_play: None }
    }

    /// Gets the [Card] played by a [Position], if any
    pub fn card_played(&self, position: Position) -> Option<Card> {
        match position {
            Position::User => self.user_play,
            Position::Dummy => self.dummy_play,
            Position::Left => self.left_play,
            Position::Right => self.right_play,
        }
    }

    /// Sets the [Card] played by a [Position]
    pub fn set_card_played(&mut self, position: Position, card: Card) {
        match position {
            Position::User => self.user_play = Some(card),
            Position::Dummy => self.dummy_play = Some(card),
            Position::Left => self.left_play = Some(card),
            Position::Right => self.right_play = Some(card),
        }
    }

    /// Returns the suit played by the lead of the current trick, if any
    pub fn lead_suit(&self) -> Option<Suit> {
        self.card_played(self.lead).map(|card| card.suit)
    }

    /// Returns an iterator over the 4 positions in this trick in order,
    /// starting with the lead position
    pub fn turn_order(&self) -> impl Iterator<Item = Position> {
        vec![self.lead, self.lead.next(), self.lead.next().next(), self.lead.next().next().next()]
            .into_iter()
    }

    /// Returns all of the positions & cards played to the current trick, in
    /// turn order
    pub fn cards(&self) -> impl Iterator<Item = (Position, Card)> + '_ {
        self.turn_order()
            .filter_map(move |position| self.card_played(position).map(|card| (position, card)))
    }

    /// Returns true if all four cards have been played to this trick
    pub fn is_completed(&self) -> bool {
        self.cards().count() == 4
    }
}

/// Represents a single game in a run
// #[derive(Debug)]
// pub struct Game {
//     pub phase: GamePhase,
//     pub trick: Trick,
//     pub trump: Option<Suit>,
//     pub user_hand: Vec<Card>,
//     pub dummy_hand: Vec<Card>,
//     pub left_opponent_hand: Vec<Card>,
//     pub right_opponet_hand: Vec<Card>,
// }

#[derive(Debug)]
pub struct Hands {
    pub user_hand: Vec<Card>,
    pub dummy_hand: Vec<Card>,
    pub left_opponent_hand: Vec<Card>,
    pub right_opponet_hand: Vec<Card>,
}

#[derive(Debug)]
pub struct GameData {
    pub hands: Hands,
}

impl GameData {
    pub fn hand(&self, position: Position) -> &Vec<Card> {
        match position {
            Position::User => &self.hands.user_hand,
            Position::Dummy => &self.hands.dummy_hand,
            Position::Left => &self.hands.left_opponent_hand,
            Position::Right => &self.hands.right_opponet_hand,
        }
    }

    pub fn hand_mut(&mut self, position: Position) -> &mut Vec<Card> {
        match position {
            Position::User => &mut self.hands.user_hand,
            Position::Dummy => &mut self.hands.dummy_hand,
            Position::Left => &mut self.hands.left_opponent_hand,
            Position::Right => &mut self.hands.right_opponet_hand,
        }
    }
}

#[derive(Debug)]
pub struct Contract {
    pub trump: Option<Suit>,
    pub tricks: usize,
    pub declarer: Position,
}

#[derive(Debug)]
pub struct PlayPhaseData {
    pub game: GameData,
    pub trick: Trick,
    pub contract: Contract,
}

#[derive(Debug)]
pub enum GamePhase {
    Auction(GameData),
    Playing(PlayPhaseData),
}

impl GamePhase {
    pub fn game(&self) -> &GameData {
        match self {
            GamePhase::Auction(game) => game,
            GamePhase::Playing(data) => &data.game,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::primitives::Rank;

    #[test]
    fn test_lead_suit() {
        assert_eq!(Trick::new(Position::Dummy).lead_suit(), None);
        let trick = Trick {
            dummy_play: Some(Card { suit: Suit::Diamonds, rank: Rank::Four }),
            ..Trick::new(Position::Dummy)
        };
        assert_eq!(trick.lead_suit(), Some(Suit::Diamonds));
    }

    #[test]
    fn test_turn_order() {
        let t = Trick::new(Position::Dummy);
        assert!(t.turn_order().eq(vec![
            Position::Dummy,
            Position::Right,
            Position::User,
            Position::Left
        ]));
    }

    #[test]
    fn test_cards() {
        let c2 = Card::new(Suit::Clubs, Rank::Two);
        let c3 = Card::new(Suit::Clubs, Rank::Three);
        let mut t = Trick::new(Position::Dummy);
        assert!(t.cards().eq(vec![]));
        t.set_card_played(Position::Dummy, c2);
        assert!(t.cards().eq(vec![(Position::Dummy, c2)]));
        t.set_card_played(Position::Right, c3);
        assert!(t.cards().eq(vec![(Position::Dummy, c2), (Position::Right, c3)]));
    }
}
