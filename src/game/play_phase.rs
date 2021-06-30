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

//! Functions for implementing the 'play'/trick-taking phase of a game

use std::{cmp::Ordering, iter};

use strum::IntoEnumIterator;

use crate::model::{
    game::{Game, Trick},
    primitives::{Card, CardId, Position},
};

/// Moves a card from a given hand into the current trick
pub fn play_card(game: &mut Game, id: CardId) {
    let card = game.hand_mut(id.position).remove(id.index);
    game.trick.set_card_played(id.position, card);
}

/// Returns all ([CardId], [Card]) pairs that the provided [Position] can currently
/// legally play, in hand order.
/// * If it is not currently the turn of this position, returns an empty
///   iterator
/// * If it this position's turn to lead *or* if this position cannot follow
///   suit, returns an iterator over all cards in hand
/// * Otherwise, returns an iterator over all cards which follow suit in the
///   current trick
pub fn legal_plays(game: &Game, position: Position) -> impl Iterator<Item = (usize, Card)> + '_ {
    let lead_suit = if let Some(lead) = game.trick.lead_suit() {
        game.hand(position)
            .iter()
            .map(|c| c.suit)
            .find(|s| *s == lead) // Do we have any cards of the lead suit?
    } else {
        None
    };

    game.hand(position)
        .iter()
        .enumerate()
        .filter(
            move |(_, card)| match (game.trick.current_turn(), lead_suit) {
                (Some(turn), _) if turn != position => false, // Not our turn
                (None, _) => false,                           // Not our turn
                (_, Some(lead)) => card.suit == lead,         // Follow lead suit
                _ => true,                                    // Lead or discard
            },
        )
        .map(|(i, c)| (i, *c))
}

/// Compares cards to determine the higher card in the context of the current
/// game, applying the trump suit and current lead suit if they are present.
/// If cards have equal power, e.g. because they are both off-suit or because
/// no cards have been yet played to the trick, returns [Ordering::Equal] even
/// if the cards themselves are distinct
pub fn compare_card_power(game: &Game, a: Card, b: Card) -> Ordering {
    match (game.trump, game.trick.lead_suit()) {
        (Some(trump), _) if a.suit == trump && b.suit == trump => a.cmp(&b),
        (Some(trump), _) if a.suit == trump => Ordering::Greater,
        (Some(trump), _) if b.suit == trump => Ordering::Less,
        (_, Some(lead)) if a.suit == lead && b.suit == lead => a.cmp(&b),
        (_, Some(lead)) if a.suit == lead => Ordering::Greater,
        (_, Some(lead)) if b.suit == lead => Ordering::Less,
        _ => Ordering::Equal,
    }
}

/// Returns a ([Position], [Card]) pair representing the position which has
/// played the highest card (as defined by the [compare_card_power] function)
/// to the current trick, or the None if no cards have been played
pub fn trick_winner(game: &Game) -> Option<(Position, Card)> {
    game.trick
        .cards()
        .max_by(|(_, a), (_, b)| compare_card_power(game, *a, *b))
}

/// Returns an iterator over all [legal_plays] for this [Position] which are
/// higher than the current [trick_winner]. If there is currently no winner,
/// result is identical to [legal_plays].
pub fn winning_plays(game: &Game, position: Position) -> impl Iterator<Item = (usize, Card)> + '_ {
    let winner = trick_winner(game);
    legal_plays(game, position).filter(move |(i, card)| {
        if let Some((_, w)) = winner {
            compare_card_power(game, *card, w) == Ordering::Greater
        } else {
            true
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        game::{deck, test_helpers},
        model::primitives::{Card, Position, Rank, Suit},
    };

    use super::*;

    #[test]
    fn test_play_card() {
        let mut g = test_helpers::create_test_game();
        assert_eq!(g.hand(Position::User)[0], test_helpers::USER_CARD_0);
        assert_eq!(g.hand(Position::User).len(), 13);
        assert_eq!(g.trick.card_played(Position::User), None);
        play_card(
            &mut g,
            CardId {
                position: Position::User,
                index: 0,
            },
        );
        assert_eq!(
            g.trick.card_played(Position::User),
            Some(test_helpers::USER_CARD_0)
        );
        assert_eq!(g.hand(Position::User).len(), 12);
    }

    #[test]
    fn test_legal_plays() {
        let mut g = test_helpers::create_test_game();
        g.trick.lead = Position::Left;
        let card = g.hand(Position::Left)[0];

        assert_eq!(legal_plays(&g, Position::Dummy).count(), 0);
        assert_eq!(legal_plays(&g, Position::Left).count(), 13);
        assert_eq!(legal_plays(&g, Position::Left).next().unwrap(), (0, card));

        let c4 = Card::new(Suit::Clubs, Rank::Four);
        let d7 = Card::new(Suit::Diamonds, Rank::Seven);
        g.trick
            .set_card_played(Position::Left, Card::new(Suit::Clubs, Rank::Two));
        g.dummy_hand = vec![c4, d7];

        assert_eq!(legal_plays(&g, Position::Left).count(), 0);
        assert!(legal_plays(&g, Position::Dummy).eq(vec![(0, c4)]));

        g.dummy_hand = vec![d7];

        assert!(legal_plays(&g, Position::Dummy).eq(vec![(0, d7)]));
    }

    #[test]
    fn test_compare_card_power() {
        let d5 = Card::new(Suit::Diamonds, Rank::Five);
        let d3 = Card::new(Suit::Diamonds, Rank::Three);
        let d8 = Card::new(Suit::Diamonds, Rank::Eight);
        let s9 = Card::new(Suit::Spades, Rank::Nine);
        let s2 = Card::new(Suit::Spades, Rank::Two);
        let h10 = Card::new(Suit::Hearts, Rank::Ten);

        let mut g = test_helpers::create_test_game();
        assert_eq!(compare_card_power(&g, d3, d3), Ordering::Equal);
        assert_eq!(compare_card_power(&g, d3, d8), Ordering::Equal);
        assert_eq!(compare_card_power(&g, d3, s9), Ordering::Equal);
        assert_eq!(compare_card_power(&g, s9, s2), Ordering::Equal);

        g.trick.lead = Position::Dummy;
        g.trick.set_card_played(Position::Dummy, d5);
        assert_eq!(compare_card_power(&g, d5, d5), Ordering::Equal);
        assert_eq!(compare_card_power(&g, d5, d3), Ordering::Greater);
        assert_eq!(compare_card_power(&g, d3, d5), Ordering::Less);
        assert_eq!(compare_card_power(&g, d5, d8), Ordering::Less);
        assert_eq!(compare_card_power(&g, d8, d5), Ordering::Greater);
        assert_eq!(compare_card_power(&g, d3, s9), Ordering::Greater);
        assert_eq!(compare_card_power(&g, s9, d3), Ordering::Less);
        assert_eq!(compare_card_power(&g, d5, h10), Ordering::Greater);
        assert_eq!(compare_card_power(&g, h10, d5), Ordering::Less);
        assert_eq!(compare_card_power(&g, h10, s9), Ordering::Equal);
        assert_eq!(compare_card_power(&g, s2, s9), Ordering::Equal);

        g.trump = Some(Suit::Spades);
        assert_eq!(compare_card_power(&g, s9, d3), Ordering::Greater);
        assert_eq!(compare_card_power(&g, d3, s9), Ordering::Less);
        assert_eq!(compare_card_power(&g, s2, d3), Ordering::Greater);
        assert_eq!(compare_card_power(&g, d3, s2), Ordering::Less);
        assert_eq!(compare_card_power(&g, d5, h10), Ordering::Greater);
        assert_eq!(compare_card_power(&g, h10, d5), Ordering::Less);
        assert_eq!(compare_card_power(&g, h10, s9), Ordering::Less);
        assert_eq!(compare_card_power(&g, s2, s9), Ordering::Less);
        assert_eq!(compare_card_power(&g, s9, s2), Ordering::Greater);
        assert_eq!(compare_card_power(&g, d5, d5), Ordering::Equal);
        assert_eq!(compare_card_power(&g, s9, s9), Ordering::Equal);
    }

    #[test]
    fn test_trick_winner() {
        let mut g = test_helpers::create_test_game();
        g.trick.lead = Position::Left;
        assert_eq!(trick_winner(&g), None);
        let c3 = Card::new(Suit::Clubs, Rank::Three);
        g.trick.set_card_played(Position::Left, c3);
        assert_eq!(trick_winner(&g), Some((Position::Left, c3)));
        let c5 = Card::new(Suit::Clubs, Rank::Five);
        g.trick.set_card_played(Position::Dummy, c5);
        assert_eq!(trick_winner(&g), Some((Position::Dummy, c5)));
        let da = Card::new(Suit::Diamonds, Rank::Ace);
        g.trick.set_card_played(Position::Right, da);
        assert_eq!(trick_winner(&g), Some((Position::Dummy, c5)));
        g.trump = Some(Suit::Hearts);
        let h2 = Card::new(Suit::Hearts, Rank::Two);
        g.trick.set_card_played(Position::User, h2);
        assert_eq!(trick_winner(&g), Some((Position::User, h2)));
    }

    #[test]
    fn test_winning_plays() {
        let mut g = test_helpers::create_test_game();
        g.trick.lead = Position::Left;
        let card = g.hand(Position::Left)[0];

        assert_eq!(winning_plays(&g, Position::Dummy).count(), 0);
        assert_eq!(winning_plays(&g, Position::Left).count(), 13);
        assert_eq!(winning_plays(&g, Position::Left).next().unwrap(), (0, card));

        let c2 = Card::new(Suit::Clubs, Rank::Two);
        let c4 = Card::new(Suit::Clubs, Rank::Four);
        let d7 = Card::new(Suit::Diamonds, Rank::Seven);
        g.trick.set_card_played(Position::Left, c2);
        g.dummy_hand = vec![c4, d7];

        assert_eq!(winning_plays(&g, Position::Left).count(), 0);
        assert!(winning_plays(&g, Position::Dummy).eq(vec![(0, c4)]));

        g.dummy_hand = vec![d7];

        assert_eq!(winning_plays(&g, Position::Dummy).count(), 0);
    }
}
