#![feature(if_let_guard)]
#![feature(let_chains)]

use context::Context;
use eval::evaluate_tome;

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
    cache: context::Context,
}

const CACHE_TITLE: &str = "_cache";

fn err<T, S: ToString>(msg: S) -> Res<T> {
    Err(msg.to_string())
}

fn parse(input: &str) -> Res<ast::Ast> {
    parser::parse(&token::tokenise(input)?)
}

fn eval(input: &str, context: &mut context::Context) -> Res<outcome::Outcome> {
    eval::evaluate(&parse(input)?, context, Context::GLOBAL_SCOPE).and_then(|oc| oc.resolved())
}

fn eval_tome(input: &str, context: &mut context::Context) -> Res<()> {
    let tokens = token::tokenise(input)?;
    let statements = parser::parse_tome(tokens)?;
    evaluate_tome(&statements, context, Context::GLOBAL_SCOPE)
}

fn interpret(input: &str, context: &mut context::Context) {
    match eval(input, context) {
        Ok(outcome) => println!("{outcome}"),
        Err(e) => println!("{e}"),
    }
}

fn load_cache(state: &mut AppState) -> Res<()> {
    if let Ok((cache, _)) = load::load(load::SaveTarget::Title(CACHE_TITLE.into())) {
        state.cache = cache;
    }

    if let Some(val) = state.cache.get_global(load::SAVE_PATH_VAR).cloned() {
        if let Err(e) = commands::load(&[], state) {
            return Err(format!("Error loading {val}: {e}"));
        }
    }

    Ok(())
}

fn main() {
    let mut state = AppState {
        input: input::Input::new(),
        context: context::Context::default(),
        interrupted: false,
        cache: context::Context::empty(),
    };

    if let Err(e) = load_cache(&mut state) {
        println!("{e}");
    }

    loop {
        match state.input.line() {
            Ok(text) => {
                if text.trim().is_empty() {
                    // ignore empty lines
                } else if text.starts_with('.') {
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
