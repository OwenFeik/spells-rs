#![feature(let_chains)]

mod input;
mod roll;
mod tracker;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args[1..].join(" ");
    match roll::roll(&input) {
        Ok(outcome) => println!("{outcome}"),
        Err(error) => println!("Failed to evaluate roll: {error}"),
    }
}
