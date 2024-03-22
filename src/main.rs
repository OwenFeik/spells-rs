#![feature(fs_try_exists)]
#![feature(let_chains)]
#![feature(split_at_checked)]

mod ast;
mod builtins;
mod commands;
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

fn interpret(input: &str, context: &mut eval::Context) {
    match eval(input, context) {
        Ok(outcome) => println!("{outcome}"),
        Err(e) => println!("{e}"),
    }
}

fn main() {
    let mut input = input::Input::new();
    let mut context = eval::Context::new();
    let mut interrupted = false;
    loop {
        match input.line() {
            Ok(text) => {
                if text.starts_with('.') {
                    if let Err(e) = commands::handle(&text) {
                        println!("{e}");
                    }
                } else {
                    interpret(&text, &mut context);
                }
            }
            Err(input::InputError::Interrupt) => {
                if interrupted {
                    std::process::exit(0);
                } else {
                    interrupted = true;
                    println!("Ctrl-C again to exit gracelessly.")
                }
            }
            Err(input::InputError::Eof) => {
                commands::exit(&[]).ok();
            }
            Err(input::InputError::Other(e)) => println!("Input error: {e}"),
        }
    }
}
