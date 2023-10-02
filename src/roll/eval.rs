use super::expr::Expr;

type EvalResult<T> = Result<T, String>;

enum Value {
    Decimal(f32),
    Natural(u32),
    Values(Vec<u32>),
}

impl Value {
    fn decimal(self) -> EvalResult<f32> {
        match self {
            Value::Decimal(v) => Ok(v),
            Value::Natural(v) => Ok(v as f32),
            Value::Values(values) => Ok(values
                .into_iter()
                .reduce(u32::saturating_add)
                .map(|v| v as f32)
                .unwrap_or(0.0)),
        }
    }
}

fn evaluate(expr: &Expr) -> Result<Value, String> {
    match expr {
        Expr::Add(lhs, rhs) => Ok(Value::Decimal(
            evaluate(lhs)?.decimal()? + evaluate(rhs)?.decimal()?,
        )),
        Expr::Sub(_, _) => todo!(),
        Expr::Mul(_, _) => todo!(),
        Expr::Div(_, _) => todo!(),
        Expr::Exp(_, _) => todo!(),
        Expr::Neg(_) => todo!(),
        Expr::Adv(_) => todo!(),
        Expr::DisAdv(_) => todo!(),
        Expr::Sort(_) => todo!(),
        Expr::Keep(_, _) => todo!(),
        Expr::Roll(q, d) => todo!(),
        Expr::Natural(_) => todo!(),
    }
}
