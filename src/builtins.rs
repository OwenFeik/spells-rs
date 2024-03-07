use crate::{err, eval, outcome::Outcome, roll::Roll, value::Value, Res};

struct GlobalFunction {
    name: &'static str,
    call: &'static dyn Fn(&GlobalFunctionCall) -> Res<Outcome>,
}

struct GlobalFunctionCall<'a> {
    gf: &'a GlobalFunction,
    args: &'a [Value],
}

impl<'a> GlobalFunctionCall<'a> {
    fn single_value(&self) -> Res<Value> {
        eval::check_argument_count(self.gf.name, 1, self.args)?;

        // Index safe because we checked length is 1.
        Ok(self.args[0].clone())
    }

    fn single_decimal(&self) -> Res<f32> {
        self.single_value().and_then(Value::decimal)
    }

    fn single_roll(&self) -> Res<Roll> {
        self.single_value().and_then(Value::roll)
    }
}

const GLOBALS: &[GlobalFunction] = &[
    GlobalFunction {
        name: "ceil",
        call: &|gfc| gfc.single_decimal().map(|v| Outcome::nat(v.ceil() as i32)),
    },
    GlobalFunction {
        name: "floor",
        call: &|gfc| gfc.single_decimal().map(|v| Outcome::nat(v.floor() as i32)),
    },
    GlobalFunction {
        name: "quantity",
        call: &|gfc| gfc.single_roll().map(|r| Outcome::nat(r.quantity as i32)),
    },
    GlobalFunction {
        name: "dice",
        call: &|gfc| gfc.single_roll().map(|r| Outcome::nat(r.die as i32)),
    },
];

pub fn call(name: &str, args: &[Value]) -> Res<Outcome> {
    for gf in GLOBALS {
        if gf.name == name {
            let gfc = GlobalFunctionCall { gf, args };
            return (gf.call)(&gfc);
        }
    }
    err(format!("Undefined function: {name}."))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ceil() {
        assert_eq!(
            call("ceil", &[Value::Decimal(2.5)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            3.0
        );
        assert_eq!(
            call("ceil", &[Value::Decimal(2.2)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            3.0
        );
        assert_eq!(
            call("ceil", &[Value::Decimal(-2.2)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            -2.0
        );
        assert!(call("ceil", &[Value::Empty]).is_err());
    }

    #[test]
    fn test_roll() {
        assert_eq!(
            call("dice", &[Value::Roll(Roll::new(8, 8))])
                .and_then(Outcome::natural)
                .map(|tup| tup.1)
                .unwrap(),
            8
        );
    }
}
