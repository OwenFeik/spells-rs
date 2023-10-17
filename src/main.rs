#![feature(let_chains)]

use input::parts;
use tracker::Tracker;

mod input;
mod roll;
mod tracker;

struct Context {
    input: input::Input,
    trackers: tracker::Tracker,
}

impl Context {
    fn new() -> Self {
        Self {
            input: input::Input::new(),
            trackers: tracker::Tracker::collection("trackers"),
        }
    }

    fn tracker(&self, name: &str) -> Option<&tracker::Tracker> {
        self.trackers.child(name)
    }
}

fn tracker(cx: &mut Context, text: &str) {
    match &parts(text)[1..] {
        [] => println!("{}", cx.trackers),
        [name] => println!("{}", cx.trackers.add(Tracker::new(name))),
        _ => println!("Usage: tracker <name>"),
    }
}

fn handle(cx: &mut Context, text: &str) {
    let command = input::command(text);
    match command {
        "tracker" => return tracker(cx, text),
        _ => {}
    }

    if let Some(tracker) = cx.tracker(command) {
        println!("{}", tracker);
        return;
    }

    match roll::roll(text) {
        Ok(roll) => println!("{roll}"),
        Err(e) => println!("Failed to parse roll: {e}"),
    }
}

fn main() {
    let mut context = Context::new();
    let mut interrupted = false;
    loop {
        match context.input.line() {
            Ok(text) => handle(&mut context, &text),
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
