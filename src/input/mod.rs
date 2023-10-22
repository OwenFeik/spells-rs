mod readline;

pub use self::readline::{Input, InputError};

pub fn command(input: &str) -> &str {
    let rest = input.trim();
    if let Some(i) = rest.find(' ') {
        rest[0..i].trim()
    } else {
        rest
    }
}

pub fn parts(input: &str) -> Vec<&str> {
    input.split(' ').filter(|s| !s.is_empty()).collect()
}

pub fn consume<'a>(input: &'a str, prefix: &str) -> &'a str {
    let rest = input.trim();
    if let Some(stripped) = rest.strip_prefix(prefix) {
        stripped.trim()
    } else {
        rest
    }
}
