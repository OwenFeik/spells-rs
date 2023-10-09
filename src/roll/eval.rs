use std::convert::TryInto;

use super::expr::Expr;

type EvalResult<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> EvalResult<T> {
    Err(msg.to_string())
}

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

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Decimal(f32),
    Natural(u32),
    Outcome(RollOutcome),
    Roll(Roll),
    Values(Vec<u32>),
}

impl Value {
    fn decimal(self) -> EvalResult<f32> {
        match self {
            Self::Decimal(v) => Ok(v),
            Self::Natural(v) => Ok(v as f32),
            Self::Roll(..) => Self::Values(self.values()?).decimal(),
            Self::Outcome(outcome) => Ok(outcome.result as f32),
            Self::Values(_) => Ok(self.natural()? as f32),
        }
    }

    fn natural(self) -> EvalResult<u32> {
        match self {
            Value::Decimal(v) => Ok(v as u32),
            Value::Natural(v) => Ok(v),
            Value::Outcome(outcome) => Ok(outcome.result),
            Value::Roll(_) => Ok(self.outcome()?.result),
            Value::Values(values) => Ok(values.iter().sum()),
        }
    }

    fn values(self) -> EvalResult<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(n) => Ok(vec![n]),
            Self::Roll(..) => Ok(self.outcome()?.values),
            Self::Outcome(outcome) => Ok(outcome.values),
            Self::Values(values) => Ok(values),
        }
    }

    fn roll(self) -> EvalResult<Roll> {
        match self {
            Value::Roll(roll) => Ok(roll),
            _ => err("Expected a roll but found non-roll."),
        }
    }

    fn outcome(self) -> EvalResult<RollOutcome> {
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
            values.iter().fold(0, |a, b| a + b)
        };

        Ok(RollOutcome {
            roll,
            values,
            result,
        })
    }
}

#[derive(Debug)]
struct ExprEval {
    value: Value,
    rolls: Vec<RollOutcome>,
}

impl ExprEval {
    fn new(value: Value) -> Self {
        Self {
            value,
            rolls: Vec::new(),
        }
    }

    fn outcome(mut self) -> EvalResult<(Self, RollOutcome)> {
        let outcome = if let Value::Outcome(outcome) = &self.value {
            outcome.clone()
        } else {
            let value = self.value.clone();
            let outcome = value.outcome()?;
            self.rolls.push(outcome.clone());
            outcome
        };
        Ok((self, outcome))
    }

    fn values(self) -> EvalResult<(Self, Vec<u32>)> {
        if matches!(self.value, Value::Roll(_)) {
            let (val, outcome) = self.outcome()?;
            Ok((val, Value::Outcome(outcome).values()?))
        } else {
            let values = self.value.clone().values()?;
            Ok((self, values))
        }
    }

    fn natural(self) -> EvalResult<(Self, u32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, values) = self.values()?;
            Ok((this, Value::Values(values).natural()?))
        } else {
            let value = self.value.clone().natural()?;
            Ok((self, value))
        }
    }

    fn decimal(self) -> EvalResult<(Self, f32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, values) = self.values()?;
            Ok((this, Value::Values(values).decimal()?))
        } else {
            let value = self.value.clone().decimal()?;
            Ok((self, value))
        }
    }

    fn unary<F: Fn(Value) -> EvalResult<Value>>(self, f: F) -> EvalResult<Self> {
        let value = f(self.value)?;
        Ok(Self {
            value,
            rolls: self.rolls,
        })
    }

    fn arithmetic<F: Fn(f32, f32) -> f32>(self, other: ExprEval, f: F) -> EvalResult<ExprEval> {
        let (mut this, lhs) = self.decimal()?;
        let (mut that, rhs) = other.decimal()?;
        this.rolls.append(&mut that.rolls);
        Ok(ExprEval {
            value: Value::Decimal(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    fn add(self, other: ExprEval) -> EvalResult<ExprEval> {
        self.arithmetic(other, |lhs, rhs| lhs + rhs)
    }

    fn sub(self, other: ExprEval) -> EvalResult<ExprEval> {
        self.arithmetic(other, |lhs, rhs| lhs - rhs)
    }

    fn mul(self, other: ExprEval) -> EvalResult<ExprEval> {
        self.arithmetic(other, |lhs, rhs| lhs * rhs)
    }

    fn div(self, other: ExprEval) -> EvalResult<ExprEval> {
        self.arithmetic(other, |lhs, rhs| lhs / rhs)
    }

    fn exp(self, other: ExprEval) -> EvalResult<ExprEval> {
        self.arithmetic(other, |lhs, rhs| lhs.powf(rhs))
    }

    fn neg(self) -> EvalResult<ExprEval> {
        self.unary(|v| v.decimal().map(Value::Decimal))
    }

    fn adv(self) -> EvalResult<ExprEval> {
        self.unary(|v| {
            v.roll().map(|mut r| {
                r.advantage = true;
                Value::Roll(r)
            })
        })
    }

    fn disadv(self) -> EvalResult<Self> {
        self.unary(|v| {
            v.roll().map(|mut r| {
                r.disadvantage = true;
                Value::Roll(r)
            })
        })
    }

    fn sort(self) -> EvalResult<Self> {
        self.unary(|v| {
            v.values().map(|mut v| {
                v.sort();
                Value::Values(v)
            })
        })
    }

    fn keep(self, rhs: Self) -> EvalResult<Self> {
        let (mut this, mut values) = self.values()?;
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
                    } else if let Some((_, sv)) = smallest && sv > *v {
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
            value: Value::Values(values),
            rolls: this.rolls,
        })
    }

    fn roll(quantity: u32, die: u32) -> Self {
        Self::new(Value::Roll(Roll::new(quantity, die)))
    }

    fn nat(value: u32) -> Self {
        Self::new(Value::Natural(value))
    }
}

fn evaluate(expr: &Expr) -> EvalResult<ExprEval> {
    match expr {
        Expr::Add(lhs, rhs) => evaluate(lhs)?.add(evaluate(rhs)?),
        Expr::Sub(lhs, rhs) => evaluate(lhs)?.sub(evaluate(rhs)?),
        Expr::Mul(lhs, rhs) => evaluate(lhs)?.mul(evaluate(rhs)?),
        Expr::Div(lhs, rhs) => evaluate(lhs)?.div(evaluate(rhs)?),
        Expr::Exp(lhs, rhs) => evaluate(lhs)?.exp(evaluate(rhs)?),
        Expr::Neg(arg) => evaluate(arg)?.neg(),
        Expr::Adv(arg) => evaluate(arg)?.adv(),
        Expr::DisAdv(arg) => evaluate(arg)?.disadv(),
        Expr::Sort(arg) => evaluate(arg)?.sort(),
        Expr::Keep(lhs, rhs) => evaluate(lhs)?.keep(evaluate(rhs)?),
        &Expr::Roll(q, d) => Ok(ExprEval::roll(q, d)),
        &Expr::Natural(v) => Ok(ExprEval::nat(v)),
    }
}

#[cfg(test)]
mod test {
    use crate::roll::{expr::Expr, parse};

    use super::*;

    #[test]
    fn test_natural() {
        let value = evaluate(&Expr::Natural(16)).unwrap().value;
        assert_eq!(value, Value::Natural(16));
    }

    #[test]
    fn test_roll() {
        let value = evaluate(&Expr::Roll(4, 12)).unwrap().value;
        assert_eq!(
            value,
            Value::Roll(Roll {
                quantity: 4,
                die: 12,
                advantage: false,
                disadvantage: false
            })
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            evaluate(&parse("5 + 4 + 3 + 2 + 1").unwrap())
                .unwrap()
                .value
                .natural()
                .unwrap(),
            5 + 4 + 3 + 2 + 1
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            evaluate(&parse("5 * 4 ^ 2 / 3 + 2 - 1").unwrap())
                .unwrap()
                .value
                .decimal()
                .unwrap(),
            5.0 * 4.0_f32.powf(2.0) / 3.0 + 2.0 - 1.0
        );
    }

    #[test]
    fn test_rolls() {
        let result = evaluate(&parse("4d6k3 + 2d4 + d20d + 2d10a").unwrap()).unwrap();
        let rolls: Vec<Roll> = result.rolls.into_iter().map(|oc| oc.roll).collect();
        assert_eq!(
            rolls,
            vec![
                Roll::new(4, 6),
                Roll::new(2, 4),
                Roll {
                    quantity: 1,
                    die: 20,
                    advantage: false,
                    disadvantage: true
                },
                Roll {
                    quantity: 2,
                    die: 10,
                    advantage: true,
                    disadvantage: false
                }
            ]
        )
    }

    #[test]
    fn test_keep() {
        let expr = ExprEval {
            value: Value::Outcome(RollOutcome {
                roll: Roll {
                    quantity: 8,
                    die: 8,
                    advantage: false,
                    disadvantage: false,
                },
                values: vec![1, 2, 3, 4, 5, 6, 7, 8],
                result: 36,
            }),
            rolls: Vec::new(),
        };
        let values = expr.keep(ExprEval::nat(6)).unwrap().value.values().unwrap();
        assert_eq!(values, vec![3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_sort() {
        let expr = ExprEval {
            value: Value::Values(vec![3, 4, 1, 7]),
            rolls: Vec::new(),
        };
        let values = expr.sort().unwrap().value.values().unwrap();
        assert_eq!(values, vec![1, 3, 4, 7]);
    }
}
