#![feature(box_syntax)]

extern crate scum;
extern crate rand;

use rand::{Rng, weak_rng};
use scum::{Player, CheatingUCT, FairUCT, ConsolePlayer, DECK, State};

fn main() {
    let mut rng = weak_rng();
    let mut deck = DECK.to_vec();
    let mut players: Vec<Box<Player>> = vec![];
    players.push(box ConsolePlayer);
    players.push(box CheatingUCT::new(100000));
    players.push(box FairUCT::new(20, 1000));
    players.push(box FairUCT::new(20, 1000));
    players.push(box FairUCT::new(20, 10000));
    rng.shuffle(&mut deck[..]);
    let mut state = State::new(players.len(), deck);
    while !state.is_terminal() {
        println!("{:?}", state);
        let player_index = state.current_player() as usize;
        println!("It's #{}'s turn:\n\t{:?}", player_index, state.top_card());
        let player = &mut players[player_index];
        let move_ = player.choose_move(state.clone());
        println!("#{} played {:?}", player_index, move_);
        state.apply(move_);
    }
    println!("\n\nWINNER: {}", state.winner());
}
