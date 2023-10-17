mod readline;

pub use self::readline::{Input, InputError};

pub fn command(input: &str) -> &str {
    if let Some(i) = input.find(' ') {
        input[0..i].trim()
    } else {
        input.trim()
    }
}

pub fn parts(input: &str) -> Vec<&str> {
    input.split(' ').filter(|s| !s.is_empty()).collect()
}
