#![feature(let_chains)]

mod roll;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = args[1..].join(" ");
    println!("{:?}", roll::parse(&input));
}
