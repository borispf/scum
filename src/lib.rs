#![feature(collections)]
#![feature(test)]

extern crate test;
#[cfg(test)] extern crate rand;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct State {
    hands: Vec<Vec<u8>>,
    discard: Vec<u8>,
    next_player: VecDeque<u8>,
    top_card: Option<(u8, u8)>,
    player_order: Vec<u8>,
    // first element is number of cards, 
}

pub const THREE: u8 = 1;
pub const FOUR : u8 = 2;
pub const FIVE : u8 = 3;
pub const SIX  : u8 = 4;
pub const SEVEN: u8 = 5;
pub const EIGHT: u8 = 6;
pub const NINE : u8 = 7;
pub const TEN  : u8 = 8;
pub const JACK : u8 = 9;
pub const QUEEN: u8 = 10;
pub const KING : u8 = 11;
pub const ACE  : u8 = 12;
pub const TWO  : u8 = 13;
pub const JOKER: u8 = 14;

pub const DECK: [u8; 54] = [
    THREE, THREE, THREE, THREE,
    FOUR,  FOUR,  FOUR,  FOUR,
    FIVE,  FIVE,  FIVE,  FIVE,
    SIX,   SIX,   SIX,   SIX,
    SEVEN, SEVEN, SEVEN, SEVEN,
    EIGHT, EIGHT, EIGHT, EIGHT,
    NINE,  NINE,  NINE,  NINE,
    TEN,   TEN,   TEN,   TEN,
    JACK,  JACK,  JACK,  JACK,
    QUEEN, QUEEN, QUEEN, QUEEN,
    KING,  KING,  KING,  KING,
    ACE,   ACE,   ACE,   ACE,
    TWO,   TWO,   TWO,   TWO,
    JOKER, JOKER,
];

impl State {
    pub fn new(num_players: usize, mut deck: Vec<u8>) -> State {
        assert!(num_players <= 54);
        let mut hands = Vec::with_capacity(num_players);
        for i in 0..num_players {
            let cards_left = deck.len();
            let num_cards = cards_left / (num_players - i);
            let mut hand = deck.split_off(cards_left - num_cards);
            hand.sort();
            hands.push(hand);
        }
        assert_eq!(0, deck.len());
        let discard = Vec::with_capacity(54);
        let next_player = (0..num_players as u8).collect();
        State {hands: hands, discard: discard, next_player: next_player,
               top_card: None, player_order: vec![]}
    }

    pub fn num_players(&self) -> usize { self.hands.len() }
    pub fn moves(&self) -> Vec<Option<(u8, u8)>> {
        let player = self.next_player.front()
            .expect("expected a next player in moves");
        let hand = &self.hands[*player as usize];
        if self.next_player.len() == 1 {
            return vec![None]
        }
        match self.top_card {
            None => all_moves(hand),
            Some((count, card)) => moves(hand, count, card),
        }
    }
    pub fn is_terminal(&self) -> bool {
        self.player_order.len() == self.num_players() - 1
    }
    pub fn apply(&mut self, muve: Option<(u8, u8)>) {
        let player = self.next_player.pop_front().expect("Ran out of players");
        if self.next_player.is_empty() {
            assert!(muve.is_none(), "state: {:?}, player: {}", &self, player);
            let num_players = self.num_players() as u8;
            self.next_player.extend(
                (player..player + num_players).map(|p| p % num_players));
            self.top_card = None;
            return;
        }
        match muve {
            Some((count, card)) => {
                assert!(self.top_card.is_none()
                    || self.top_card.unwrap().0 == count
                    || card == JOKER);
                assert!(self.top_card.is_none()
                    || self.top_card.unwrap().1 < card);
                self.next_player.push_back(player);
                for _ in 0..count {
                    self.play_card(player, card);
                }
                if self.hands[player as usize].is_empty() {
                    self.player_order.push(player);
                }
                self.top_card = muve
            },
            None => {},  // Pass, no nothing.
        }
    }
    fn play_card(&mut self, player: u8, card: u8) {
        let hand = &mut self.hands[player as usize];
        let pos = hand.iter().position(|c| *c == card)
            .expect("couldn't find card");
        hand.remove(pos);
        self.discard.push(card);
    }
}

fn moves(hand: &Vec<u8>, count: u8, card: u8) -> Vec<Option<(u8, u8)>> {
    let mut moves = Vec::with_capacity(hand.len() / count as usize + 2);
    moves.push(None);
    let mut i = hand.len();
    let offset = count as usize - 1;
    let mut has_joker = false;
    while i > 0 {
        i -= 1;
        if hand[i] <= card {
            break;
        }
        if hand[i] == JOKER {
            if !has_joker {
                moves.push(Some((1, JOKER)));
                has_joker = true;
            }
            continue;
        }
        if i < offset {
            break;
        }
        if hand[i] == hand[i - offset] {
            let card = hand[i];
            moves.push(Some((count, card)));
            i -= offset;
            while i > 1 && hand[i] == card { i -= 1; }
        }
    }
    moves
}

fn all_moves(hand: &Vec<u8>) -> Vec<Option<(u8, u8)>> {
    if hand.is_empty() {
        return vec![None]
    }
    let mut moves = vec![];
    let mut i = 0;
    let end = hand.len();
    while i < end {
        if hand[i] == JOKER {
            moves.push(Some((1, JOKER)));
            break;
        }
        for off in 0..5 {
            let j = i + off;
            if j < end && hand[i] == hand[j] {
                moves.push(Some((off as u8 + 1, hand[i])));
            } else {
                i = j;
                break;
            }
        }
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{all_moves, moves};

    #[allow(non_snake_case)]
    fn M(count: u8, card: u8) -> Option<(u8, u8)> { Some((count, card)) }

    #[test]
    fn test_moves() {
        let hand = vec![THREE, FOUR, FOUR, FOUR, FOUR, JOKER, JOKER];

        assert_eq!(
            vec![None, M(1, JOKER), M(1, FOUR)], moves(&hand, 1, THREE));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 1, FOUR));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 1, FIVE));

        assert_eq!(
            vec![None, M(1, JOKER), M(2, FOUR)], moves(&hand, 2, THREE));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 2, FOUR));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 2, FIVE));

        assert_eq!(
            vec![None, M(1, JOKER), M(3, FOUR)], moves(&hand, 3, THREE));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 3, FOUR));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 3, FIVE));

        assert_eq!(
            vec![None, M(1, JOKER), M(4, FOUR)], moves(&hand, 4, THREE));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 4, FOUR));
        assert_eq!(vec![None, M(1, JOKER)], moves(&hand, 4, FIVE));
    }

    #[test]
    fn test_all_moves() {
        let hand = vec![THREE, FOUR, FOUR, FOUR, FOUR, JOKER, JOKER];

        assert_eq!(vec![
            M(1, THREE),
            M(1, FOUR), M(2, FOUR), M(3, FOUR), M(4, FOUR),
            M(1, JOKER)
        ], all_moves(&hand));
    }

    #[test]
    fn test_num_players() {
        for i in 1..55 {
            assert_eq!(i, State::new(i, DECK.to_vec()).num_players());
        }
    }

    #[test]
    fn test_new() {
        let state = State::new(5, DECK.to_vec());
        assert_eq!(10, state.hands[0].len());
        assert_eq!(11, state.hands[1].len());
        assert_eq!(11, state.hands[2].len());
        assert_eq!(11, state.hands[3].len());
        assert_eq!(11, state.hands[4].len());

        let mut all_cards = vec![];
        all_cards = all_cards + &state.hands[0][..] +
            &state.hands[1][..] + &state.hands[2][..] +
            &state.hands[3][..] + &state.hands[4][..];
        all_cards.sort();
        assert_eq!(all_cards, DECK.to_vec());
    }

    #[test]
    fn smoke_test() {
        let mut state = State::new(4, DECK.to_vec());
        while !state.is_terminal() {
            let action = state.moves().pop().expect(
                &(format!("State didn't produce moves: {:?}", state)));
            state.apply(action);
        }
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    use super::{all_moves, moves};

    use rand::{Rng, weak_rng};
    use test::Bencher;

    #[bench]
    fn bench_all_moves(b: &mut Bencher) {
        let hand = vec![
            THREE,
            FOUR, FOUR, FOUR, FOUR,
            FIVE, FIVE, FIVE,
            JOKER, JOKER];
        b.iter(|| {
            all_moves(&hand)
        });
    }

    #[bench]
    fn bench_moves_1(b: &mut Bencher) {
        let hand = vec![
            THREE,
            FOUR, FOUR, FOUR, FOUR,
            FIVE, FIVE, FIVE,
            JOKER, JOKER];
        b.iter(|| {
            moves(&hand, 1, THREE)
        });
    }

    #[bench]
    fn bench_moves_2(b: &mut Bencher) {
        let hand = vec![
            THREE,
            FOUR, FOUR, FOUR, FOUR,
            FIVE, FIVE, FIVE,
            JOKER, JOKER];
        b.iter(|| {
            moves(&hand, 2, THREE)
        });
    }

    #[bench]
    fn bench_moves_3(b: &mut Bencher) {
        let hand = vec![
            THREE,
            FOUR, FOUR, FOUR, FOUR,
            FIVE, FIVE, FIVE,
            JOKER, JOKER];
        b.iter(|| {
            moves(&hand, 3, THREE)
        });
    }

    #[bench]
    fn bench_moves_4(b: &mut Bencher) {
        let hand = vec![
            THREE,
            FOUR, FOUR, FOUR, FOUR,
            FIVE, FIVE, FIVE,
            JOKER, JOKER];
        b.iter(|| {
            moves(&hand, 4, THREE)
        });
    }

    #[bench]
    fn bench_random_game(b: &mut Bencher) {
        let mut rng = weak_rng();
        b.iter(|| {
            let mut deck = DECK.to_vec();
            rng.shuffle(&mut deck[..]);
            let mut state = State::new(5, deck);
            while !state.is_terminal() {
                let action = *rng.choose(&mut state.moves()[..]).unwrap();
                state.apply(action);
            }
        });
    }
}
