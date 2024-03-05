use std::{convert::TryInto, fmt::Display};

use crate::{err, Res};

use super::{Roll, RollOutcome};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Decimal(f32),
    Natural(u32),
    Outcome(RollOutcome),
    Roll(Roll),
    Values(Vec<u32>),
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
            Self::Empty => err("Empty cannot be interpreted as decimal."),
        }
    }

    pub fn natural(self) -> Res<u32> {
        match self {
            Self::Decimal(v) => Ok(v as u32),
            Self::Natural(v) => Ok(v),
            Self::Outcome(outcome) => Ok(outcome.result),
            Self::Roll(_) => Ok(self.outcome()?.result),
            Self::Values(values) => Ok(values.iter().sum()),
            Self::Empty => err("Empty cannot be interpreted as natural."),
        }
    }

    pub fn values(self) -> Res<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(n) => Ok(vec![n]),
            Self::Roll(..) => Ok(self.outcome()?.values),
            Self::Outcome(outcome) => Ok(outcome.values),
            Self::Values(values) => Ok(values),
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
            &Value::Decimal(v) => {
                // If value is an integer, skip decimal places. Else round to 2 places.
                let string_value = if v as i32 as f32 == v {
                    (v as i32).to_string()
                } else {
                    format!("{:.2}", v)
                };
                write!(f, "{string_value}")
            }
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
                        .join(", ")
                )
            }
            Value::Empty => write!(f, "()"),
        }
    }
}
