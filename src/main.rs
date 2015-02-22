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
    while !state.is_terminal() {
        let iters = if state.current_player() == 0 { 10000 } else { 10000 };
        let move_ = best_move(&mut state.clone(), iters, &mut rng);
        if state.top_card().is_none() {
            println!("\n");
        }
        println!("{} plays {:?}  [{:?}]",
            state.current_player(), move_, &state);
        state.apply(move_);
    }
    println!("\n\nWINNER: {}", state.winner());
}
