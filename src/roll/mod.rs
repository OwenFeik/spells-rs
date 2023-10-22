use std::fmt::{Display, Write};

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

impl Display for Roll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quantity = if self.quantity == 1 {
            "".to_string()
        } else {
            self.quantity.to_string()
        };

        let die = self.die;

        let advstr = if self.advantage && !self.disadvantage {
            "a"
        } else if self.disadvantage && !self.advantage {
            "d"
        } else {
            ""
        };

        write!(f, "{quantity}d{die}{advstr}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RollOutcome {
    roll: Roll,
    values: Vec<u32>,
    result: u32,
}

impl Display for RollOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\tRolls: \t{}\tTotal: {}",
            self.roll,
            self.values
                .iter()
                .map(u32::to_string)
                .collect::<Vec<String>>()
                .join(", "),
            self.result
        )
    }
}

#[derive(Debug)]
pub struct Outcome {
    value: f32,
    rolls: Vec<RollOutcome>,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for roll in &self.rolls {
            roll.fmt(f)?;
            f.write_char('\n')?;
        }

        // If value is an integer, skip decimal places. Else round to 2 places.
        let string_value = if self.value as i32 as f32 == self.value {
            (self.value as i32).to_string()
        } else {
            format!("{:.2}", self.value)
        };
        write!(f, "Grand Total: {string_value}")
    }
}

pub struct Statement(expr::Ast);

impl Statement {
    pub fn eval(&self) -> Result<Outcome, String> {
        eval::eval(&self.0)
    }
}

pub fn parse(input: &str) -> Result<Statement, String> {
    Ok(Statement(expr::lex(&token::tokenise(input))?))
}

pub fn roll(input: &str) -> Result<Outcome, String> {
    parse(input)?.eval()
}
