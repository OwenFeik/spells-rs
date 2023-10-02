use std::convert::TryInto;

use super::expr::Expr;

type EvalResult<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> EvalResult<T> {
    Err(msg.to_string())
}

type Roll = (u32, u32);
type RollOutcome = (Roll, Vec<u32>);

enum Value {
    Decimal(f32),
    Natural(u32),
    Roll(Roll),
    Values(Vec<u32>),
}

impl Value {
    fn decimal(self) -> EvalResult<f32> {
        match self {
            Self::Decimal(v) => Ok(v),
            Self::Natural(v) => Ok(v as f32),
            Self::Roll(..) => Self::Values(self.values()?).decimal(),
            Self::Values(values) => Ok(values
                .into_iter()
                .reduce(u32::saturating_add)
                .map(|v| v as f32)
                .unwrap_or(0.0)),
        }
    }

    fn values(self) -> EvalResult<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(n) => Ok(vec![n]),
            Self::Roll((q, d)) => {
                let quantity = q.try_into().map_err(|_| format!("{q} is too many dice."))?;
                let mut values = Vec::with_capacity(quantity);
                for _ in 0..quantity {
                    values.push(rand::Rng::gen_range(&mut rand::thread_rng(), 1..=d))
                }
                Ok(values)
            }
            Self::Values(values) => Ok(values),
        }
    }

    fn roll(self) -> EvalResult<Roll> {
        match self {
            Value::Roll(roll) => Ok(roll),
            _ => err("Expected a roll but found non-roll."),
        }
    }
}

struct ExprEval {
    value: Value,
    rolls: Vec<Roll>,
}

impl ExprEval {
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
        let value = Value::Decimal(-self.value.decimal()?);
        Ok(ExprEval {
            value,
            rolls: self.rolls,
        })
    }
}

fn evaluate(expr: &Expr) -> EvalResult<ExprEval> {
    match expr {
        Expr::Add(lhs, rhs) => evaluate(lhs)?.add(evaluate(rhs)?),
        Expr::Sub(lhs, rhs) => evaluate(lhs)?.sub(evaluate(rhs)?),
        Expr::Mul(lhs, rhs) => evaluate(lhs)?.mul(evaluate(rhs)?),
        Expr::Div(lhs, rhs) => evaluate(lhs)?.div(evaluate(rhs)?),
        Expr::Exp(lhs, rhs) => evaluate(lhs)?.exp(evaluate(rhs)?),
        Expr::Neg(val) => evaluate(val)?.neg(),
        Expr::Adv(_) => todo!(),
        Expr::DisAdv(_) => todo!(),
        Expr::Sort(_) => todo!(),
        Expr::Keep(_, _) => todo!(),
        Expr::Roll(q, d) => todo!(),
        Expr::Natural(_) => todo!(),
    }
}
