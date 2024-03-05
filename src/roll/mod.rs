use std::fmt::{Display, Write};

mod ast;
mod eval;
mod token;
mod value;

pub use self::eval::Context;
use self::value::Value;

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
    value: Value,
    rolls: Vec<RollOutcome>,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for roll in &self.rolls {
            roll.fmt(f)?;
            f.write_char('\n')?;
        }

        if matches!(self.value, Value::Empty) {
            std::fmt::Result::Ok(())
        } else {
            write!(f, "{}", self.value)
        }
    }
}
pub struct Statement(ast::Ast);

impl Statement {
    pub fn eval(&self, context: &mut Context) -> Result<Outcome, String> {
        eval::eval(&self.0, context)
    }
}

pub fn parse(input: &str) -> Result<Statement, String> {
    Ok(Statement(ast::lex(&token::tokenise(input)?)?))
}
