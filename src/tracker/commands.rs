use crate::{input, Context};

use super::Tracker;

pub fn handle(cx: &mut Context, text: &str) {
    match input::command(text) {
        "" => cx.trackers.print(),
        name => {
            if let Some(tracker) = cx.tracker(name) {
                tracker.handle(input::consume(text, name));
            } else {
                cx.trackers.add(Tracker::new(name));
                println!("Created new tracker {name}.");
            }
        }
    }
}
