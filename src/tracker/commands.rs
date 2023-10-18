use crate::{input, Context};

use super::Tracker;

pub fn handle(cx: &mut Context, text: &str) {
    match &input::parts(text)[1..] {
        [] => println!("{}", cx.trackers),
        [name] => {
            if let Some(tracker) = cx.trackers.child(name) {
                tracker.print();
            } else {
                cx.trackers.add(Tracker::new(name));
                println!("Created new tracker {name}.");
            }
        }
        _ => println!("Usage: tracker <name>"),
    }
}
