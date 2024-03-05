#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![allow(unused)]

mod input;
mod roll;

fn main() {
    let mut input = input::Input::new();
    let mut context = roll::Context::new();
    let mut interrupted = false;
    loop {
        match input.line() {
            Ok(text) => match roll::parse(&text) {
                Ok(statement) => println!("{:?}", statement.eval()),
                Err(e) => println!("{e}"),
            },
            Err(input::InputError::Interrupt) => {
                if interrupted {
                    std::process::exit(0);
                } else {
                    interrupted = true;
                    println!("Ctrl-C again to exit gracelessly.")
                }
            }
            Err(input::InputError::Eof) => break,
            Err(input::InputError::Other(e)) => println!("Input error: {e}"),
        }
    }
}
