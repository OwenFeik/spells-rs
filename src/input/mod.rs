mod readline;

pub fn word(input: &str) -> &str {
    if let Some(i) = input.find(' ') {
        input[0..i].trim()
    } else {
        input.trim()
    }
}
