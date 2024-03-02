#![feature(iterator_try_collect)]
#![feature(let_chains)]
#![allow(unused)]

mod input;
mod roll;

pub struct Context {
    input: input::Input,
}

impl Context {
    fn new() -> Self {
        Self {
            input: input::Input::new(),
        }
    }
}

fn handle(cx: &mut Context, text: &str) {}

fn main() {
    let mut context = Context::new();
    let mut interrupted = false;
    loop {
        match context.input.line() {
            Ok(text) => {
                println!();
                handle(&mut context, &text);
            }
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
