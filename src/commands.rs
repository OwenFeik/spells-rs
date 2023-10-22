use super::Context;
use crate::{input, tracker};

pub fn handle(cx: &mut Context, text: &str) -> bool {
    match input::command(text) {
        "tracker" => tracker::handle(cx, input::consume(text, "tracker")),
        _ => return false,
    }
    true
}
