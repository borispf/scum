extern crate env_logger;
extern crate scum;
extern crate rand;

use rand::{Rng, weak_rng};
use scum::{best_move, DECK, State};

fn main() {
    env_logger::init().unwrap();

    let mut rng = weak_rng();
    let mut deck = DECK.to_vec();
    rng.shuffle(&mut deck[..]);
    let mut state = State::new(4, deck);
    println!("{:?}", state);
    while !state.is_terminal() {
        if state.top_card().is_none() {
            println!("\n");
        }
        let move_ = best_move(
            &mut state.to_partial_state(), 50, 1000, &mut rng);
        println!("{} => {:13}  [{:?}]",
            state.current_player(), format!("{:?}", move_), &state);
        state.apply(move_);
    }
    println!("\n\nWINNER: {}", state.winner());
}
