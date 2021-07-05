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

use anyhow::Result;
use strum::IntoEnumIterator;

use crate::{
    agents::agent::{self, Agent},
    model::{
        game::{PlayPhaseData, Trick},
        primitives::{Card, CardId, Position},
        state::State,
    },
};

/// Moves a card from a given hand into the current trick
pub fn play_card(data: &mut PlayPhaseData, id: CardId) {
    let card = data.game.hand_mut(id.position).remove(id.index);
    data.trick.set_card_played(id.position, card);
}

/// Returns the next player to act in the play phase
/// * If the game is over (hands are empty), returns None
/// * If the current trick is empty, returns the lead player
/// * If the current trick is in-progress, returns the next player in turn order
/// * If the current trick is full, returns the winner of that trick
pub fn next_to_play(data: &PlayPhaseData) -> Option<Position> {
    let next = data.trick.turn_order().find(|p| data.trick.card_played(*p).is_none());
    match next {
        _ if Position::iter().all(|p| data.game.hand(p).is_empty()) => None,
        None => trick_winner(data).map(|(p, _)| p),
        Some(n) => Some(n),
    }
}

/// Returns all ([CardId], [Card]) pairs that the provided [Position] can
/// currently legally play, in hand order.
/// * If it is not currently the turn of this position, returns an empty
///   iterator
/// * If it this position's turn to lead *or* if this position cannot follow
///   suit, returns an iterator over all cards in hand
/// * Otherwise, returns an iterator over all cards which follow suit in the
///   current trick
pub fn legal_plays(
    data: &PlayPhaseData,
    position: Position,
) -> impl Iterator<Item = (usize, Card)> + '_ {
    let lead_suit = if data.trick.is_completed() {
        None
    } else {
        data.trick.lead_suit().and_then(|lead| {
            // Do we have any cards of the lead suit?
            data.game.hand(position).iter().map(|c| c.suit).find(|s| *s == lead)
        })
    };

    data.game
        .hand(position)
        .iter()
        .enumerate()
        .filter(move |(_, card)| match (next_to_play(data), lead_suit) {
            (Some(turn), _) if turn != position => false, // Not our turn
            (None, _) => false,                           // Game is over
            (_, Some(lead)) => card.suit == lead,         // Follow lead suit
            _ => true,                                    // Lead or discard
        })
        .map(|(i, c)| (i, *c))
}

/// Compares cards to determine the higher card in the context of the current
/// game, applying the trump suit and current lead suit if they are present.
/// If cards have equal power, e.g. because they are both off-suit or because
/// no cards have been yet played to the trick, returns [Ordering::Equal] even
/// if the cards themselves are distinct
pub fn compare_card_power(data: &PlayPhaseData, a: Card, b: Card) -> Ordering {
    match (data.contract.trump, data.trick.lead_suit()) {
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
pub fn trick_winner(data: &PlayPhaseData) -> Option<(Position, Card)> {
    data.trick.cards().max_by(|(_, a), (_, b)| compare_card_power(data, *a, *b))
}

/// Returns an iterator over all [legal_plays] for this [Position] which are
/// higher in power than the current [trick_winner]. If there is currently no
/// winner, result is identical to [legal_plays].
pub fn winning_plays(
    data: &PlayPhaseData,
    position: Position,
) -> impl Iterator<Item = (usize, Card)> + '_ {
    let winner = trick_winner(data);
    legal_plays(data, position).filter(move |(i, card)| {
        if let Some((_, w)) = winner {
            compare_card_power(data, *card, w) == Ordering::Greater
        } else {
            true
        }
    })
}

/// Plays the card with the provided [CardId] and then advances the game state
/// by invoking the current Agent for its action (if required) and updating
/// the score. If the current trick is full before invoking this action, it is
/// cleared and this card is set as the leader of a new trick.
pub fn resolve_card_play_action(
    data: &mut PlayPhaseData,
    agent: &dyn Agent,
    id: CardId,
) -> Result<()> {
    if data.trick.is_completed() {
        data.trick = Trick::new(id.position);
    }

    play_card(data, id);

    if !data.trick.is_completed() {
        let next = id.position.next();
        assert!(next.is_agent());
        let agent_action = agent.select_play(data, next);
        play_card(data, CardId::new(next, agent_action));
    }

    Ok(())
}

/// Clears the current Trick and sets the winner as the leader of a new Trick,
/// and then invokes the current Agent for its action (if required).
pub fn resolve_continue_action(data: &mut PlayPhaseData, agent: &dyn Agent) -> Result<()> {
    let (winner, _) = trick_winner(data).expect("No trick winner");
    data.trick = Trick::new(winner);

    if winner.is_agent() {
        let agent_action = agent.select_play(data, winner);
        play_card(data, CardId::new(winner, agent_action));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        game::{self, deck, test_helpers},
        model::{
            game::GamePhase,
            primitives::{Card, Position, Rank, Suit},
        },
    };

    #[test]
    fn test_play_card() {
        let mut g = test_helpers::create_test_play_phase();
        assert_eq!(g.game.hand(Position::User)[0], test_helpers::USER_CARD_0);
        assert_eq!(g.game.hand(Position::User).len(), 13);
        assert_eq!(g.trick.card_played(Position::User), None);
        play_card(&mut g, CardId { position: Position::User, index: 0 });
        assert_eq!(g.trick.card_played(Position::User), Some(test_helpers::USER_CARD_0));
        assert_eq!(g.game.hand(Position::User).len(), 12);
    }

    #[test]
    fn test_next_to_play() {
        let mut g = test_helpers::create_test_play_phase();
        g.trick = Trick::new(Position::Dummy);
        assert_eq!(next_to_play(&g), Some(Position::Dummy));
        g.trick.set_card_played(Position::Dummy, Card::new(Suit::Clubs, Rank::Two));
        assert_eq!(next_to_play(&g), Some(Position::Right));
        g.trick.set_card_played(Position::Right, Card::new(Suit::Clubs, Rank::Three));
        g.trick.set_card_played(Position::User, Card::new(Suit::Clubs, Rank::Ace));
        assert_eq!(next_to_play(&g), Some(Position::Left));
        g.trick.set_card_played(Position::Left, Card::new(Suit::Clubs, Rank::Five));
        assert_eq!(next_to_play(&g), Some(Position::User));

        let mut game_over = test_helpers::create_empty_game();
        assert_eq!(next_to_play(&game_over), None);

        game_over.game.hands.right_opponet_hand.push(Card::new(Suit::Clubs, Rank::Three));
        game_over.trick.set_card_played(Position::Dummy, Card::new(Suit::Clubs, Rank::Two));
        game_over.trick.set_card_played(Position::User, Card::new(Suit::Clubs, Rank::Ace));
        game_over.trick.set_card_played(Position::Left, Card::new(Suit::Clubs, Rank::Five));
        assert_eq!(next_to_play(&game_over), Some(Position::Right));
    }

    #[test]
    fn test_legal_plays() {
        let mut g = test_helpers::create_test_play_phase();
        g.trick.lead = Position::Left;
        let card = g.game.hand(Position::Left)[0];

        assert_eq!(legal_plays(&g, Position::Dummy).count(), 0);
        assert_eq!(legal_plays(&g, Position::Left).count(), 13);
        assert_eq!(legal_plays(&g, Position::Left).next().unwrap(), (0, card));

        let c4 = Card::new(Suit::Clubs, Rank::Four);
        let d7 = Card::new(Suit::Diamonds, Rank::Seven);
        g.trick.set_card_played(Position::Left, Card::new(Suit::Clubs, Rank::Two));
        g.game.hands.dummy_hand = vec![c4, d7];

        assert_eq!(legal_plays(&g, Position::Left).count(), 0);
        assert!(legal_plays(&g, Position::Dummy).eq(vec![(0, c4)]));

        g.game.hands.dummy_hand = vec![d7];

        assert!(legal_plays(&g, Position::Dummy).eq(vec![(0, d7)]));

        g.trick.set_card_played(Position::Dummy, d7);
        g.trick.set_card_played(Position::Right, c4);

        // After the 4th card is played, the winner (right) can play any card
        assert_eq!(legal_plays(&g, Position::Right).count(), 0);
        g.trick.set_card_played(Position::User, Card::new(Suit::Clubs, Rank::Three));
        assert_eq!(legal_plays(&g, Position::Right).count(), 13);
        assert_eq!(legal_plays(&g, Position::Left).count(), 0);
        assert_eq!(legal_plays(&g, Position::User).count(), 0);
        assert_eq!(legal_plays(&g, Position::Dummy).count(), 0);
    }

    #[test]
    fn test_legal_plays_end_of_game() {
        let mut g = test_helpers::create_empty_game();
        g.trick.lead = Position::User;
        let c3 = Card::new(Suit::Clubs, Rank::Three);
        let c4 = Card::new(Suit::Clubs, Rank::Four);
        let d10 = Card::new(Suit::Diamonds, Rank::Ten);
        let c6 = Card::new(Suit::Clubs, Rank::Six);
        g.game.hands.user_hand.push(c3);
        g.game.hands.left_opponent_hand.push(c4);
        g.game.hands.dummy_hand.push(d10);
        g.game.hands.right_opponet_hand.push(c6);

        assert!(legal_plays(&g, Position::User).eq(vec![(0, c3)]));
        assert_eq!(legal_plays(&g, Position::Dummy).count(), 0);

        g.trick.set_card_played(Position::User, g.game.hands.user_hand.pop().unwrap());
        g.trick.set_card_played(Position::Left, g.game.hands.left_opponent_hand.pop().unwrap());

        assert_eq!(legal_plays(&g, Position::User).count(), 0);
        assert!(legal_plays(&g, Position::Dummy).eq(vec![(0, d10)]));
    }

    #[test]
    fn test_compare_card_power() {
        let d5 = Card::new(Suit::Diamonds, Rank::Five);
        let d3 = Card::new(Suit::Diamonds, Rank::Three);
        let d8 = Card::new(Suit::Diamonds, Rank::Eight);
        let s9 = Card::new(Suit::Spades, Rank::Nine);
        let s2 = Card::new(Suit::Spades, Rank::Two);
        let h10 = Card::new(Suit::Hearts, Rank::Ten);

        let mut g = test_helpers::create_test_play_phase();
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

        g.contract.trump = Some(Suit::Spades);
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
        let mut g = test_helpers::create_test_play_phase();
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
        g.contract.trump = Some(Suit::Hearts);
        let h2 = Card::new(Suit::Hearts, Rank::Two);
        g.trick.set_card_played(Position::User, h2);
        assert_eq!(trick_winner(&g), Some((Position::User, h2)));
    }

    #[test]
    fn test_winning_plays() {
        let mut g = test_helpers::create_test_play_phase();
        g.trick.lead = Position::Left;
        let card = g.game.hand(Position::Left)[0];

        assert_eq!(winning_plays(&g, Position::Dummy).count(), 0);
        assert_eq!(winning_plays(&g, Position::Left).count(), 13);
        assert_eq!(winning_plays(&g, Position::Left).next().unwrap(), (0, card));

        let c2 = Card::new(Suit::Clubs, Rank::Two);
        let c4 = Card::new(Suit::Clubs, Rank::Four);
        let d7 = Card::new(Suit::Diamonds, Rank::Seven);
        g.trick.set_card_played(Position::Left, c2);
        g.game.hands.dummy_hand = vec![c4, d7];

        assert_eq!(winning_plays(&g, Position::Left).count(), 0);
        assert!(winning_plays(&g, Position::Dummy).eq(vec![(0, c4)]));

        g.game.hands.dummy_hand = vec![d7];

        assert_eq!(winning_plays(&g, Position::Dummy).count(), 0);
    }

    #[test]
    fn test_resolve_card_play_action() {
        let mut data = test_helpers::create_test_play_phase();
        let agent = test_helpers::create_test_agent();

        assert!(
            resolve_card_play_action(&mut data, &*agent, CardId::new(Position::User, 0)).is_ok()
        );
        assert_eq!(
            data.trick.card_played(Position::User).unwrap(),
            Card::new(Suit::Clubs, Rank::Two)
        );
        assert_eq!(
            data.trick.card_played(Position::Left).unwrap(),
            Card::new(Suit::Clubs, Rank::Four)
        );

        assert!(
            resolve_card_play_action(&mut data, &*agent, CardId::new(Position::Dummy, 4)).is_ok()
        );
        assert_eq!(
            data.trick.card_played(Position::Dummy).unwrap(),
            Card::new(Suit::Clubs, Rank::Five)
        );
        assert_eq!(
            data.trick.card_played(Position::Right).unwrap(),
            Card::new(Suit::Clubs, Rank::Three)
        );

        assert!(
            resolve_card_play_action(&mut data, &*agent, CardId::new(Position::User, 11)).is_ok()
        );
        assert_eq!(
            data.trick.card_played(Position::User).unwrap(),
            Card::new(Suit::Spades, Rank::King)
        );
        assert_eq!(
            data.trick.card_played(Position::Left).unwrap(),
            Card::new(Suit::Spades, Rank::Three)
        );
        assert!(data.trick.card_played(Position::Dummy).is_none());
        assert!(data.trick.card_played(Position::Right).is_none());
    }

    #[test]
    fn test_resolve_continue_action() {
        let mut data = test_helpers::create_test_play_phase();
        let agent = test_helpers::create_test_agent();

        data.trick.set_card_played(Position::User, Card::new(Suit::Spades, Rank::Two));
        data.trick.set_card_played(Position::Left, Card::new(Suit::Spades, Rank::Three));
        data.trick.set_card_played(Position::Dummy, Card::new(Suit::Hearts, Rank::Ace));
        data.trick.set_card_played(Position::Right, Card::new(Suit::Spades, Rank::Five));

        assert!(resolve_continue_action(&mut data, &*agent).is_ok());

        assert_eq!(
            data.trick.card_played(Position::Right).unwrap(),
            Card::new(Suit::Diamonds, Rank::Four)
        );
        assert!(data.trick.card_played(Position::Dummy).is_none());
        assert!(data.trick.card_played(Position::User).is_none());
    }
}
