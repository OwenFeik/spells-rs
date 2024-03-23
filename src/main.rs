#![feature(fs_try_exists)]
#![feature(let_chains)]
#![feature(split_at_checked)]

mod ast;
mod builtins;
mod commands;
mod context;
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

struct AppState {
    input: input::Input,
    context: context::Context,
    interrupted: bool,
    title: Option<String>,
}

fn err<T, S: ToString>(msg: S) -> Res<T> {
    Err(msg.to_string())
}

fn parse(input: &str) -> Res<ast::Ast> {
    parser::parse(&token::tokenise(input)?)
}

fn eval(input: &str, context: &mut context::Context) -> Res<outcome::Outcome> {
    eval::evaluate(&parse(input)?, context).and_then(|oc| oc.resolved())
}

fn interpret(input: &str, context: &mut context::Context) {
    match eval(input, context) {
        Ok(outcome) => println!("{outcome}"),
        Err(e) => println!("{e}"),
    }
}

fn main() {
    let mut state = AppState {
        input: input::Input::new(),
        context: context::Context::default(),
        interrupted: false,
        title: None,
    };

    loop {
        match state.input.line() {
            Ok(text) => {
                if text.starts_with('.') {
                    if let Err(e) = commands::handle(&text, &mut state) {
                        println!("{e}");
                    }
                } else {
                    interpret(&text, &mut state.context);
                }
            }
            Err(input::InputError::Interrupt) => {
                if state.interrupted {
                    std::process::exit(0);
                } else {
                    state.interrupted = true;
                    println!("Ctrl-C again to exit gracelessly.")
                }
            }
            Err(input::InputError::Eof) => {
                commands::exit(&[], &mut state).ok();
            }
            Err(input::InputError::Other(e)) => println!("Input error: {e}"),
        }
    }
}
