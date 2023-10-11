mod eval;
mod expr;
mod token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roll {
    quantity: u32,
    die: u32,
    advantage: bool,
    disadvantage: bool,
}

impl Roll {
    fn new(quantity: u32, die: u32) -> Self {
        Roll {
            quantity,
            die,
            advantage: false,
            disadvantage: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollOutcome {
    roll: Roll,
    values: Vec<u32>,
    result: u32,
}

#[derive(Debug)]
pub struct Outcome {
    value: f32,
    rolls: Vec<RollOutcome>,
}

pub fn roll(input: &str) -> Result<Outcome, String> {
    eval::eval(&expr::lex(&token::tokenise(input))?)
}
