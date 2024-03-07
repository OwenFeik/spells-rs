#![feature(let_chains)]

mod ast;
mod eval;
mod input;
mod outcome;
mod parser;
mod roll;
mod token;
mod value;

type Res<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> Res<T> {
    Err(msg.to_string())
}

fn evaluate(input: &str, context: &mut eval::Context) -> Res<outcome::Outcome> {
    let tokens = token::tokenise(input)?;
    let ast = parser::parse(&tokens)?;
    eval::eval(&ast, context)
}

fn main() {
    let mut input = input::Input::new();
    let mut context = eval::Context::new();
    let mut interrupted = false;
    loop {
        match input.line() {
            Ok(text) => match evaluate(&text, &mut context) {
                Ok(outcome) => println!("{outcome}"),
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
