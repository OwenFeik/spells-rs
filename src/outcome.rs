use std::fmt::{Display, Write};

use crate::{roll::RollOutcome, value::Value, Res};

#[derive(Debug, PartialEq)]
pub struct Outcome {
    pub value: Value,
    pub rolls: Vec<RollOutcome>,
}

impl Outcome {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            rolls: Vec::new(),
        }
    }

    fn resolve_for<T, F: Fn(Value) -> Res<T>>(mut self, f: F) -> Res<(Self, T)> {
        if matches!(self.value, Value::Roll(_)) {
            let outcome = self.value.outcome()?;
            self.value = Value::Outcome(outcome.clone());
            self.rolls.push(outcome);
        }
        let value = f(self.value.clone())?;
        Ok((self, value))
    }

    pub fn rolls(self) -> Res<(Self, Vec<u64>)> {
        self.resolve_for(Value::rolls)
    }

    pub fn natural(self) -> Res<(Self, i64)> {
        self.resolve_for(Value::natural)
    }

    pub fn decimal(self) -> Res<(Self, f64)> {
        self.resolve_for(Value::decimal)
    }

    pub fn bool(self) -> Res<(Self, bool)> {
        self.resolve_for(Value::bool)
    }

    fn arithmetic<F: Fn(f64, f64) -> f64>(self, other: Outcome, f: F) -> Res<Outcome> {
        let (mut this, lhs) = self.decimal()?;
        let (mut that, rhs) = other.decimal()?;
        this.rolls.append(&mut that.rolls);
        Ok(Outcome {
            value: Value::Decimal(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    fn numeric_comparison<F: Fn(f64, f64) -> bool>(self, other: Outcome, f: F) -> Res<Outcome> {
        let (mut this, lhs) = self.decimal()?;
        let (mut that, rhs) = other.decimal()?;
        this.rolls.append(&mut that.rolls);
        Ok(Outcome {
            value: Value::Bool(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    fn boolean<F: Fn(bool, bool) -> bool>(self, other: Outcome, f: F) -> Res<Outcome> {
        let (mut this, lhs) = self.bool()?;
        let (mut that, rhs) = other.bool()?;
        this.rolls.append(&mut that.rolls);
        Ok(Outcome {
            value: Value::Bool(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    pub fn add(mut self, mut other: Outcome) -> Res<Outcome> {
        if matches!(self.value, Value::String(..)) || matches!(other.value, Value::String(..)) {
            let lhs = self.value.string()?;
            let rhs = other.value.string()?;

            self.rolls.append(&mut other.rolls);
            Ok(Outcome {
                value: Value::String(format!("{lhs}{rhs}")),
                rolls: self.rolls,
            })
        } else {
            self.arithmetic(other, |lhs, rhs| lhs + rhs)
        }
    }

    pub fn sub(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs - rhs)
    }

    pub fn mul(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs * rhs)
    }

    pub fn div(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs / rhs)
    }

    pub fn exp(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs.powf(rhs))
    }

    pub fn neg(self) -> Res<Outcome> {
        let (this, value) = self.decimal()?;
        Ok(Self {
            value: Value::Decimal(-value),
            rolls: this.rolls,
        })
    }

    pub fn adv(self) -> Res<Outcome> {
        let mut roll = self.value.roll()?;
        roll.advantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    pub fn disadv(self) -> Res<Self> {
        let mut roll = self.value.roll()?;
        roll.disadvantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    pub fn keep(self, rhs: Self) -> Res<Self> {
        let (mut this, mut values) = self.rolls()?;
        let (mut that, keep) = rhs.natural()?;
        this.rolls.append(&mut that.rolls);

        let keep = keep as usize;
        if keep < values.len() {
            let mut to_remove = values.len() - keep;
            let mut smallest = None;
            while to_remove > 0 {
                for (i, v) in values.iter().enumerate() {
                    if smallest.is_none() {
                        smallest = Some((i, *v));
                    } else if let Some((_, sv)) = smallest
                        && sv > *v
                    {
                        smallest = Some((i, *v));
                    }
                }

                if let Some((i, _)) = smallest {
                    values.remove(i);
                }
                to_remove -= 1;
            }
        }

        Ok(Self {
            value: Value::Rolls(values),
            rolls: this.rolls,
        })
    }

    pub fn greater_than(self, rhs: Self) -> Res<Self> {
        self.numeric_comparison(rhs, |a, b| a > b)
    }

    pub fn greater_equal(self, rhs: Self) -> Res<Self> {
        self.numeric_comparison(rhs, |a, b| a >= b)
    }

    pub fn less_than(self, rhs: Self) -> Res<Self> {
        self.numeric_comparison(rhs, |a, b| a < b)
    }

    pub fn less_equal(self, rhs: Self) -> Res<Self> {
        self.numeric_comparison(rhs, |a, b| a <= b)
    }

    pub fn equal(mut self, mut other: Self) -> Res<Self> {
        self.rolls.append(&mut other.rolls);
        Ok(Self {
            value: Value::Bool(self.value == other.value),
            rolls: self.rolls,
        })
    }

    pub fn and(self, other: Self) -> Res<Self> {
        self.boolean(other, |a, b| a && b)
    }

    pub fn or(self, other: Self) -> Res<Self> {
        self.boolean(other, |a, b| a || b)
    }

    pub fn not(mut self) -> Res<Self> {
        self.value = Value::Bool(!self.value.bool()?);
        Ok(self)
    }

    pub fn nat(value: i64) -> Self {
        Self::new(Value::Natural(value))
    }

    pub fn empty() -> Self {
        Self::new(Value::Empty)
    }

    pub fn resolved(self) -> Res<Self> {
        if matches!(self.value, Value::Roll(_)) {
            self.natural().map(|oc| oc.0)
        } else {
            Ok(self)
        }
    }
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
