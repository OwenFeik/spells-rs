#![feature(fs_try_exists)]
#![feature(let_chains)]

mod ast;
mod builtins;
mod eval;
mod input;
mod load;
mod operator;
mod outcome;
mod parser;
mod roll;
mod token;
mod value;

type Res<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> Res<T> {
    Err(msg.to_string())
}

fn parse(input: &str) -> Res<ast::Ast> {
    parser::parse(&token::tokenise(input)?)
}

fn eval(input: &str, context: &mut eval::Context) -> Res<outcome::Outcome> {
    eval::eval_roll(&parse(input)?, context)
}

fn main() {
    let mut input = input::Input::new();
    let mut context = eval::Context::new();
    if let Err(e) = load::load(&mut context) {
        println!("{e}");
    }

    let mut interrupted = false;
    loop {
        match input.line() {
            Ok(text) => match eval(&text, &mut context) {
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
            Err(input::InputError::Eof) => {
                if let Err(e) = load::save(&mut context) {
                    println!("{e}");
                }
                break;
            }
            Err(input::InputError::Other(e)) => println!("Input error: {e}"),
        }
    }
}
