use std::fmt::{Display, Write};

use crate::{roll::RollOutcome, value::Value, Res};

#[derive(Debug)]
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

    pub fn outcome(mut self) -> Res<(Self, RollOutcome)> {
        let outcome = if let Value::Outcome(outcome) = &self.value {
            outcome.clone()
        } else {
            let outcome = self.value.outcome()?;
            self.value = Value::Outcome(outcome.clone());
            self.rolls.push(outcome.clone());
            outcome
        };
        Ok((self, outcome))
    }

    pub fn rolls(self) -> Res<(Self, Vec<u32>)> {
        if matches!(self.value, Value::Roll(_)) {
            let (val, outcome) = self.outcome()?;
            Ok((val, Value::Outcome(outcome).rolls()?))
        } else {
            let values = self.value.clone().rolls()?;
            Ok((self, values))
        }
    }

    pub fn natural(self) -> Res<(Self, i32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, outcome) = self.outcome()?;
            Ok((this, outcome.result as i32))
        } else {
            let value = self.value.clone().natural()?;
            Ok((self, value))
        }
    }

    pub fn decimal(self) -> Res<(Self, f32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, outcome) = self.outcome()?;
            Ok((this, outcome.result as f32))
        } else {
            let value = self.value.clone().decimal()?;
            Ok((self, value))
        }
    }

    fn arithmetic<F: Fn(f32, f32) -> f32>(self, other: Outcome, f: F) -> Res<Outcome> {
        let (mut this, lhs) = self.decimal()?;
        let (mut that, rhs) = other.decimal()?;
        this.rolls.append(&mut that.rolls);
        Ok(Outcome {
            value: Value::Decimal(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    pub fn add(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs + rhs)
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

    pub fn sort(self) -> Res<Self> {
        let (this, mut values) = self.rolls()?;
        values.sort();
        Ok(Self {
            value: Value::Rolls(values),
            rolls: this.rolls,
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

    pub fn nat(value: i32) -> Self {
        Self::new(Value::Natural(value))
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
