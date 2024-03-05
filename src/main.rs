#![feature(let_chains)]

mod input;
mod roll;

type Res<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> Res<T> {
    Err(msg.to_string())
}

fn main() {
    let mut input = input::Input::new();
    let mut context = roll::Context::new();
    let mut interrupted = false;
    loop {
        match input.line() {
            Ok(text) => match roll::parse(&text).and_then(|s| s.eval(&mut context)) {
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
