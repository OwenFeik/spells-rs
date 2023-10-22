#![feature(let_chains)]

mod commands;
mod input;
mod roll;
mod tracker;

pub struct Context {
    input: input::Input,
    trackers: tracker::Tracker,
}

impl Context {
    fn new() -> Self {
        Self {
            input: input::Input::new(),
            trackers: tracker::Tracker::new("trackers"),
        }
    }

    fn tracker(&self, name: &str) -> Option<&tracker::Tracker> {
        self.trackers.get(name)
    }
}

fn handle(cx: &mut Context, text: &str) {
    if commands::handle(cx, text) {
    } else if let Some(tracker) = cx.tracker(input::command(text)) {
        tracker.handle(input::consume(text, tracker.name()));
    } else {
        match roll::roll(text) {
            Ok(roll) => println!("{roll}"),
            Err(e) => println!("Failed to parse roll: {e}"),
        }
    }
}

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
