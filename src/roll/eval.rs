use std::convert::TryInto;

use super::expr::Expr;

type EvalResult<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> EvalResult<T> {
    Err(msg.to_string())
}

struct Roll {
    quantity: u32,
    die: u32,
    advantage: bool,
    disadvantage: bool,
}

type RollOutcome = (Roll, Vec<u32>, u32);

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
            Self::Outcome((roll, values, result)) => Ok(result as f32),
            Self::Values(values) => Ok(self.natural()? as f32),
        }
    }

    fn natural(self) -> EvalResult<u32> {
        match self {
            Value::Decimal(v) => Ok(v as u32),
            Value::Natural(v) => Ok(v),
            Value::Outcome((roll, values, result)) => Ok(result),
            Value::Roll(_) => Ok(self.outcome()?.2),
            Value::Values(values) => Ok(values.iter().fold(0, |a, b| a + b)),
        }
    }

    fn values(self) -> EvalResult<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(n) => Ok(vec![n]),
            Self::Roll(Roll { quantity, die, .. }) => {
                let (_, values, _) = self.outcome()?;
                Ok(values)
            }
            Self::Outcome((roll, values, result)) => Ok(values),
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

        Ok((roll, values, result))
    }
}

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
        match &self.value {
            Value::Decimal(_) => err("Expected a roll but found decimal."),
            Value::Natural(_) => err("Expected a roll but found natural."),
            Value::Roll(_) => {
                let outcome = self.value.outcome()?;
                self.rolls.push(outcome);
                Ok((self, outcome))
            }
            &Value::Outcome(outcome) => Ok((self, outcome)),
            Value::Values(_) => err("Expected a roll but found values."),
        }
    }

    fn values(self) -> EvalResult<(Self, Vec<u32>)> {
        match self.value {
            Value::Outcome(_) | Value::Roll(_) => {
                let (val, outcome) = self.outcome()?;
                let values = Value::Outcome(outcome).values()?;
                Ok((val, values))
            }
            _ => Ok((self, self.value.values()?)),
        }
    }

    fn unary<F: Fn(Value) -> EvalResult<Value>>(mut self, f: F) -> EvalResult<Self> {
        let value = f(self.value)?;
        Ok(Self {
            value,
            rolls: self.rolls,
        })
    }

    fn arithmetic<F: Fn(f32, f32) -> f32>(
        mut self,
        mut other: ExprEval,
        f: F,
    ) -> EvalResult<ExprEval> {
        self.rolls.append(&mut other.rolls);
        let rolls = self.rolls;
        let lhs = self.value.decimal()?;
        let rhs = other.value.decimal()?;
        Ok(ExprEval {
            value: Value::Decimal(f(lhs, rhs)),
            rolls,
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

    fn keep(self, mut rhs: Self) -> EvalResult<Self> {
        let (mut this, mut values) = self.values()?;
        let keep = rhs.value.natural()? as usize;
        this.rolls.append(&mut rhs.rolls);

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
            rolls: self.rolls,
        })
    }

    fn roll(quantity: u32, die: u32) -> Self {
        Self::new(Value::Roll(Roll {
            quantity,
            die,
            advantage: false,
            disadvantage: false,
        }))
    }

    fn natural(value: u32) -> Self {
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
        Expr::DisAdv(arg) => evaluate(arg)?.adv(),
        Expr::Sort(arg) => evaluate(arg)?.sort(),
        Expr::Keep(lhs, rhs) => evaluate(lhs)?.keep(evaluate(rhs)?),
        &Expr::Roll(q, d) => Ok(ExprEval::roll(q, d)),
        &Expr::Natural(v) => Ok(ExprEval::natural(v)),
    }
}
