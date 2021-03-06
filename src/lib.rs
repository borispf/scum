#![crate_name(scum)]
#![feature(collections)]
#![feature(core)]
#![feature(old_io)]
#![feature(std_misc)]
#![feature(test)]

extern crate test;
extern crate rand;
#[macro_use]
extern crate log;

use rand::{Rng, XorShiftRng};
use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fmt::Write;
use std::num::Float;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct State {
    hands: Vec<Hand>,
    top_card: Move,
    discard: Vec<u8>,
    next_player: VecDeque<u8>,
    finishing_order: Vec<u8>,
}

pub struct PartialState {
    player: u8,
    hand_sizes: Vec<usize>,
    hand: Hand,
    discard: Vec<u8>,
    next_player: VecDeque<u8>,
    top_card: Move,
    finishing_order: Vec<u8>,
}

// first element is number of cards,
pub type Move = Option<(u8, u8)>;
pub type Hand = Vec<u8>;

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
               top_card: None, finishing_order: vec![]}
    }

    pub fn realisation_from<R: Rng>(ps: &PartialState, rng: &mut R) -> State {
        let mut deck = DECK.to_vec();
        for card in ps.hand.iter().chain(ps.discard.iter()) {
            let pos = deck.iter().position(|c| *c == *card)
                .expect("couldn't find card2");
            deck.remove(pos);
        }
        assert_eq!(deck.len(), 54 - ps.discard.len() - ps.hand.len());
        rng.shuffle(&mut deck);
        let mut hands = Vec::with_capacity(ps.hand_sizes.len());
        // println!("{:?}", ps.hand_sizes);
        for (i, size) in ps.hand_sizes.iter().enumerate() {
            if i == ps.player as usize {
                hands.push(ps.hand.clone());
            } else {
                let cards_left = deck.len();
                // println!("{:?} {:?}", size, deck.len());
                let mut hand = deck.split_off(cards_left - *size);
                hand.sort();
                hands.push(hand);
            }
        }
        assert_eq!(0, deck.len());
        State {
            hands: hands,
            discard: ps.discard.clone(),
            next_player: ps.next_player.clone(),
            top_card: ps.top_card.clone(),
            finishing_order: ps.finishing_order.clone(),
        }
    }
    pub fn to_partial_state(&self) -> PartialState {
        let hand_sizes = self.hands.iter().map(|h| h.len()).collect();
        let player = self.current_player();
        PartialState {
            player: player,
            hand_sizes: hand_sizes,
            hand: self.hands[player as usize].clone(),
            discard: self.discard.clone(),
            next_player: self.next_player.clone(),
            top_card: self.top_card.clone(),
            finishing_order: self.finishing_order.clone(),
        }
    }

    pub fn num_players(&self) -> usize { self.hands.len() }
    pub fn moves(&self) -> Vec<Move> {
        let player = self.next_player.front()
            .expect("expected a next player in moves");
        let hand = &self.hands[*player as usize];
        if self.is_terminal() {
            return vec![]
        }
        match self.top_card {
            None => all_moves(hand),
            Some((count, card)) => moves(hand, count, card),
        }
    }
    pub fn is_terminal(&self) -> bool {
        !self.finishing_order.is_empty()
    }
    pub fn current_player(&self) -> u8 {
        *self.next_player.front()
                    .expect("expected a next player in current_player")
    }
    pub fn winner(&self) -> u8 { self.finishing_order[0] }
    pub fn apply(&mut self, muve: Move) {
        let player = self.next_player.pop_front().expect("Ran out of players");
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
                    self.finishing_order.push(player);
                }
                self.top_card = muve
            },
            None => {},  // Pass, no nothing.
        }
        if self.next_player.len() == 1 {
            assert!(muve.is_none(), "state: {:?}, player: {}", &self, player);
            let player = self.next_player.pop_front().unwrap();
            let num_players = self.num_players() as u8;
            self.next_player.extend(
                (player..player + num_players).map(|p| p % num_players));
            self.top_card = None;
            return;
        }
    }
    fn play_card(&mut self, player: u8, card: u8) {
        let hand = &mut self.hands[player as usize];
        // println!("{:?} {:?} {:?}", player, card, hand);
        let pos = hand.iter().position(|c| *c == card)
            .expect("couldn't find card");
        hand.remove(pos);
        self.discard.push(card);
    }
    pub fn top_card<'a>(&'a self) -> &'a Move { &self.top_card }
}

pub fn play_randomly<R>(state: &mut State, rng: &mut R) where R: Rng {
    while !state.is_terminal() {
        let action = *rng.choose(&mut state.moves()[..]).unwrap();
        state.apply(action);
    }
}

fn moves(hand: &Hand, count: u8, card: u8) -> Vec<Move> {
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
            while i > 0 && hand[i - 1] == card { i -= 1; }
        }
    }
    moves
}

fn all_moves(hand: &Hand) -> Vec<Move> {
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

const NOBODY: u8 = -1;
pub struct Node {
    children: Vec<(Move, Node)>,
    untried_moves: Vec<Move>,
    player: u8,
    wins: f64,
    plays: f64,
}

const UCTK: f64 = 0.7;

impl Node {
    pub fn new(player: u8, untried_moves: Vec<Move>) -> Node {
        Node {children: vec![], untried_moves: untried_moves,
            player: player, plays: 0., wins: 0.}
    }

    pub fn select_child(&self) -> usize {
        (0..self.children.len()).max_by(|i| {
            let (_, ref c) = self.children[*i];
            ((c.wins / c.plays + UCTK * (self.plays.ln() / c.plays).sqrt()) *
            1000000.) as i64
        }).unwrap()
    }

    pub fn add_child<R:Rng>(&mut self, state: &mut State, rng: &mut R) {
        let player = state.current_player();
        let move_ = self.untried_moves.pop()
            .expect("tried to pop untried move");
        state.apply(move_);
        let mut moves = state.moves();
        rng.shuffle(&mut moves);
        let mut node = Node::new(player, moves);
        play_randomly(state, rng);
        node.update(state);
        self.children.push((move_, node));
    }

    pub fn update(&mut self, state: &mut State) {
        self.plays += 1.;
        if state.winner() == self.player {
            self.wins += 1.;
        }
    }

    pub fn uct<R: Rng>(&mut self, state: &mut State, rng: &mut R) {
        if self.untried_moves.is_empty() && !self.children.is_empty() {
            let i = self.select_child();
            let &mut (move_, ref mut child) = &mut self.children[i];
            assert_eq!(child.player, state.current_player());
            state.apply(move_);
            child.uct(state, rng);
        } else if !self.untried_moves.is_empty() {
            self.add_child(state, rng);
        }
        self.update(state);
    }

    pub fn tree_string(&self) -> String {
        let mut str = String::new();
        self.write_tree(0, None, &mut str);
        str
    }

    #[allow(unused_must_use)]
    fn write_tree<W: Write>(&self, indent: usize, m: Move, w: &mut W) {
        let indent_string = self.indent_string(indent);
        write!(w, "\n{}{:?}: [P:{} W/P:{}/{} U:{:?}]",
            indent_string, m, self.player, self.wins as usize,
            self.plays as usize, self.untried_moves);
        for &(move_, ref child) in self.children.iter() {
            child.write_tree(indent + 1, move_, w);
        }
    }
    fn indent_string(&self, indent: usize) -> String {
        let mut str = String::with_capacity(2 * indent);
        for _ in 0..indent {
            str.push_str("| ");
        }
        str
    }
}

pub fn best_move<R: Rng>(
    partial: &PartialState, reals: usize, iters: usize, rng: &mut R) -> Move {

    let mut outcomes: HashMap<Move, usize> = HashMap::new();
    for r in 0..reals {
        let state = State::realisation_from(partial, rng);
        let mut moves = state.moves();
        if moves.len() == 1 {
            return moves[0];
        }
        rng.shuffle(&mut moves);
        let mut root = Node::new(NOBODY, moves);
        for _ in 0..iters {
            root.uct(&mut state.clone(), rng);
        }
        for &(ref move_, ref node) in root.children.iter() {
            match outcomes.entry(*move_) {
                Occupied(mut o) => {
                    let old = *o.get();
                    o.insert(old + node.plays as usize);
                },
                Vacant(v) => {
                    v.insert(node.plays as usize);
                },
            }
        }
        if r == 0 && partial.hand.len() <= 3 {
            debug!("{}", root.tree_string());
        }
    }
    if partial.hand.len() <= 3 {
        debug!("{:?}", outcomes);
    }
    *outcomes.iter().max_by(|c| *c.1 as i64).unwrap().0
}

pub trait Player {
    fn choose_move(&mut self, s: State) -> Move;
}

pub trait FairPlayer {
    fn choose_move(&mut self, p: PartialState) -> Move;
}

impl<T: FairPlayer> Player for T {
    fn choose_move(&mut self, s: State) -> Move {
        self.choose_move(s.to_partial_state())
    }
}

pub struct CheatingUCT {
    rng: XorShiftRng,
    iters: usize,
}

use rand::weak_rng;

impl CheatingUCT {
    pub fn new(iters: usize) -> CheatingUCT {
        CheatingUCT {rng: weak_rng(), iters: iters}
    }
}

impl Player for CheatingUCT {
    fn choose_move(&mut self, s: State) -> Move {
        let mut moves = s.moves();
        if moves.len() == 1 {
            return moves[0];
        }
        self.rng.shuffle(&mut moves);
        let mut root = Node::new(NOBODY, moves);
        for _ in 0..self.iters {
            root.uct(&mut s.clone(), &mut self.rng);
        }
        root.children.iter().max_by(|c| c.1.plays as usize).unwrap().0
    }
}

pub struct FairUCT {
    rng: XorShiftRng,
    reals: usize,
    iters: usize,
}

impl FairUCT {
    pub fn new(reals: usize, iters: usize) -> FairUCT {
        FairUCT {rng: weak_rng(), reals: reals, iters: iters}
    }
}

impl FairPlayer for FairUCT {
    fn choose_move(&mut self, p: PartialState) -> Move {
        best_move(&p, self.reals, self.iters, &mut self.rng)
    }
}

pub struct ConsolePlayer;

use std::old_io;

impl FairPlayer for ConsolePlayer {
    fn choose_move(&mut self, p: PartialState) -> Move {
        static CARDS: [&'static str; 15] = ["",
            "3", "4", "5", "6", "7", "8", "9", "10", "Jack",
            "Queen", "King", "Ace", "2", "Joker"];

        match p.top_card {
            Some((count, card)) =>
                println!("Top card: {}x {}", count, CARDS[card as usize]),
            None => println!("Play whatever you want :)"),
        };
        print!("Your cards:");
        for c in p.hand.iter() {
            print!(" {}", CARDS[*c as usize]);
        } println!("");
        // Dirty hack for getting the moves.
        let mut move_ = None;
        while {
            println!("Possible Moves:");
            let moves = State::realisation_from(&p, &mut rand::weak_rng()).moves();
            for (i, m) in moves.iter().enumerate() {
                match m {
                    &Some((count, card)) =>
                        println!("{}: {}x {}", i, count, CARDS[card as usize]),
                    &None => println!("{}: Pass", i),
                }
            }
            println!("INPUT:");
            let mut reader = old_io::stdin();
            let input = reader.read_line().ok().expect("Failed to read line");
            println!("YOU TYPED:");
            println!("{}", input);
            let res = FromStr::from_str(input.trim())
                .map_err(|e| format!("{:?}", e))
                .and_then(|i| moves.get(i).map(|m| move_ = *m)
                    .ok_or("Not a valid move".to_string()))
                .map_err(|err| println!("{:?}", err));
            res.is_err()
        } {}
        move_
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{all_moves, moves};

    #[allow(non_snake_case)]
    fn M(count: u8, card: u8) -> Move { Some((count, card)) }

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
    // Regression test from playing with the test client.
    fn test_moves_close() {
        let hand = vec![1, 9, 11, 12, 13];
        let top_card = 11;
        assert_eq!(vec![None, Some((1, 13)), Some((1, 12))],
            moves(&hand, 1, top_card));
    }

    #[test]
    // Regression test from playing with the test client.
    fn test_moves_duplicate_move() {
        let hand = vec![7, 7, 9, 10, 11, 12, 13];
        let top_card = 4;
        assert_eq!(vec![None, Some((1, 13)), Some((1, 12)), Some((1, 11)),
            Some((1, 10)), Some((1, 9)), Some((1, 7))],
            moves(&hand, 1, top_card));
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
            play_randomly(&mut state, &mut rng);
            state.winner()
        });
    }

    #[bench]
    fn bench_uct(b: &mut Bencher) {
        let mut rng = weak_rng();
        let mut deck = DECK.to_vec();
        rng.shuffle(&mut deck[..]);
        let state = State::new(5, deck);
        let mut root = Node::new(state.current_player(), state.moves());
        b.iter(|| {
            root.uct(&mut state.clone(), &mut rng);
            root.plays
        });
    }
}
