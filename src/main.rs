#![feature(inherent_associated_types)]

mod roll;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args[1..].join(" ");
}
