use std::{convert::TryInto, fmt::Display};

use crate::{
    err,
    roll::{Roll, RollOutcome},
    Res,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Decimal(f64),
    Natural(i64),
    Outcome(RollOutcome),
    Roll(Roll),
    Rolls(Vec<u64>),
    List(Vec<Value>),
    String(String),
    Empty,
}

impl Value {
    pub fn bool(self) -> Res<bool> {
        match self {
            Value::Bool(v) => Ok(v),
            Value::Natural(n) => Ok(n != 0),
            Value::List(vs) => Ok(!vs.is_empty()),
            Value::String(s) => Ok(!s.is_empty()),
            _ => Err(format!("{self} cannot be interpreted as a bool.")),
        }
    }

    pub fn decimal(self) -> Res<f64> {
        match self {
            Self::Decimal(v) => Ok(v),
            Self::Natural(v) => Ok(v as f64),
            Self::Roll(..) => Self::Rolls(self.rolls()?).decimal(),
            Self::Rolls(rolls) => Ok(rolls.iter().sum::<u64>() as f64),
            Self::Outcome(outcome) => Ok(outcome.result as f64),
            Self::List(values) => {
                let mut total = 0.0;
                for value in values {
                    total += value.decimal()?;
                }
                Ok(total)
            }
            Self::Bool(v) => Err(format!("{v} cannot be interpreted as decimal.")),
            Self::String(_) => err("String cannot be interpreted as decimal."),
            Self::Empty => err("Empty cannot be interpreted as decimal."),
        }
    }

    pub fn natural(self) -> Res<i64> {
        match self {
            Self::Decimal(v) => Ok(v as i64),
            Self::Natural(v) => Ok(v),
            Self::Outcome(outcome) => Ok(outcome.result as i64),
            Self::Roll(_) => Ok(self.outcome()?.result as i64),
            Self::Rolls(rolls) => Ok(rolls.iter().sum::<u64>() as i64),
            Self::List(values) => {
                let mut total = 0;
                for value in values {
                    total += value.natural()?;
                }
                Ok(total)
            }
            Self::Bool(v) => Err(format!("{v} cannot be interpreted as natural.")),
            Self::String(_) => err("String cannot be interpreted as natural."),
            Self::Empty => err("Empty cannot be interpreted as natural."),
        }
    }

    pub fn rolls(self) -> Res<Vec<u64>> {
        match self {
            Self::Bool(v) => Err(format!("{v} cannot be interpreted as rolls.")),
            Self::Decimal(_) => err("Decimal value cannot be interpreted as rolls."),
            Self::Natural(_) => err("Natural value cannot be interpreted as rolls."),
            Self::Roll(..) => Value::Outcome(self.outcome()?).rolls(),
            Self::Rolls(rolls) => Ok(rolls),
            Self::Outcome(outcome) => Ok(outcome.rolls),
            Self::List(_) => err("List cannot be interpreted as rolls."),
            Self::String(_) => err("String cannot be interpreted as rolls."),
            Self::Empty => err("Empty cannot be interpreted as rolls."),
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
            rolls: values,
            result,
        })
    }

    pub fn string(self) -> Res<String> {
        match self {
            Value::String(string) => Ok(string),
            _ => Err(format!("{self} cannot be interpreted as a string.")),
        }
    }

    pub fn list(self) -> Res<Vec<Self>> {
        match self {
            Value::String(string) => Ok(string
                .chars()
                .map(|c| Value::String(c.to_string()))
                .collect()),
            Value::List(values) => Ok(values),
            Value::Roll(..) | Value::Rolls(..) | Value::Outcome(..) => Ok(self
                .rolls()?
                .iter()
                .map(|v| Self::Natural(*v as i64))
                .collect()),
            _ => Err(format!("{self} cannot be interpreted as a list.")),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Value::Bool(v) => write!(f, "{v}"),
            &Value::Decimal(v) => write!(f, "{}", (v * 100.0).round() / 100.0), // 2 places.
            Value::Natural(v) => write!(f, "{v}"),
            Value::Outcome(v) => write!(f, "{}", v.result),
            Value::Roll(v) => write!(f, "{v}"),
            Value::Rolls(rolls) => {
                write!(
                    f,
                    "[{}]",
                    rolls
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::List(values) => {
                write!(
                    f,
                    "[{}]",
                    values
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Value::String(s) => write!(f, r#""{}""#, s.replace('"', "\\\"")),
            Value::Empty => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{context::Context, eval};

    use super::*;

    fn test_homoiconicity(val: Value) {
        let mut cx = Context::empty();
        assert_eq!(eval(&val.to_string(), &mut cx).unwrap().value, val);
    }

    #[test]
    fn test_quotes_escaped() {
        test_homoiconicity(Value::String("\"quoted\"".into()));
    }

    #[test]
    fn test_list() {
        test_homoiconicity(Value::List(vec![
            Value::String("abc".into()),
            Value::Natural(1),
            Value::Roll(Roll::new(8, 8)),
        ]));
    }

    #[test]
    fn test_string_as_list() {
        assert_eq!(
            Value::String("abc".into()).list(),
            Ok(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into())
            ])
        )
    }
}
