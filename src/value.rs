use std::{convert::TryInto, fmt::Display};

use crate::{
    err,
    roll::{Roll, RollOutcome},
    Res,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Decimal(f32),
    Natural(i32),
    Outcome(RollOutcome),
    Roll(Roll),
    Values(Vec<u32>),
    String(String),
    Empty,
}

impl Value {
    pub fn decimal(self) -> Res<f32> {
        match self {
            Self::Decimal(v) => Ok(v),
            Self::Natural(v) => Ok(v as f32),
            Self::Roll(..) => Self::Values(self.values()?).decimal(),
            Self::Outcome(outcome) => Ok(outcome.result as f32),
            Self::Values(_) => Ok(self.natural()? as f32),
            Self::String(_) => err("String cannot be interpreted as decimal."),
            Self::Empty => err("Empty cannot be interpreted as decimal."),
        }
    }

    pub fn natural(self) -> Res<i32> {
        match self {
            Self::Decimal(v) => Ok(v as i32),
            Self::Natural(v) => Ok(v),
            Self::Outcome(outcome) => Ok(outcome.result as i32),
            Self::Roll(_) => Ok(self.outcome()?.result as i32),
            Self::Values(values) => Ok(values.iter().sum::<u32>() as i32),
            Self::String(_) => err("String cannot be interpreted as natural."),
            Self::Empty => err("Empty cannot be interpreted as natural."),
        }
    }

    pub fn values(self) -> Res<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(_) => err("Natural value cannot be interpreted as values."),
            Self::Roll(..) => Ok(self.outcome()?.values),
            Self::Outcome(outcome) => Ok(outcome.values),
            Self::Values(values) => Ok(values),
            Self::String(_) => err("String cannot be interpreted as values."),
            Self::Empty => err("Empty cannot be interpreted as values."),
        }
    }

    pub fn roll(self) -> Res<Roll> {
        match self {
            Value::Roll(roll) => Ok(roll),
            _ => err("Expected a roll but found non-roll."),
        }
    }

    pub fn outcome(self) -> Res<RollOutcome> {
        if let Value::Outcome(outcome) = self {
            return Ok(outcome);
        }

        let roll = self.roll()?;
        let mut quantity: usize = roll
            .quantity
            .try_into()
            .map_err(|_| format!("{} is too many dice.", roll.quantity))?;
        if roll.advantage ^ roll.disadvantage {
            quantity = quantity.max(2);
        }

        let mut values = Vec::with_capacity(quantity);
        let die = roll.die;
        for _ in 0..quantity {
            values.push(rand::Rng::gen_range(&mut rand::thread_rng(), 1..=die))
        }

        let result = if roll.advantage ^ roll.disadvantage {
            let mut sorted = values.clone();
            sorted.sort();

            // Safe because quantity.max(2)
            if roll.advantage {
                *values.last().unwrap()
            } else {
                *values.first().unwrap()
            }
        } else {
            values.iter().sum()
        };

        Ok(RollOutcome {
            roll,
            values,
            result,
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Value::Decimal(v) => write!(f, "{}", (v * 100.0).round() / 100.0), // 2 places.
            Value::Natural(v) => write!(f, "{v}"),
            Value::Outcome(v) => write!(f, "{}", v.result),
            Value::Roll(v) => write!(f, "{v}"),
            Value::Values(values) => {
                write!(
                    f,
                    "[{}]",
                    values
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
            Value::String(s) => write!(f, r#""{}""#, s.replace('"', "\\\"")),
            Value::Empty => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::context::Context;

    use super::*;

    #[test]
    fn test_quotes_escaped() {
        let mut context = Context::empty();
        let value = Value::String("\"quoted\"".into());
        assert_eq!(
            crate::eval(&value.to_string(), &mut context).unwrap().value,
            value
        );
    }
}
