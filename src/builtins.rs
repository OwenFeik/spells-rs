use crate::{err, eval, outcome::Outcome, roll::Roll, value::Value, Res};

pub const DEFAULT_GLOBALS: &[&str] = &[
    "STRENGTH = 10",
    "DEXTERITY = 10",
    "CONSTITUTION = 10",
    "INTELLIGENCE = 10",
    "WISDOM = 10",
    "CHARISMA = 10",
    "modifier(stat) = floor((stat - 10) / 2)",
    "STR() = modifier(STRENGTH)",
    "DEX() = modifier(DEXTERITY)",
    "CON() = modifier(CONSTITUTION)",
    "INT() = modifier(INTELLIGENCE)",
    "WIS() = modifier(WISDOM)",
    "CHA() = modifier(CHARISMA)",
    "LEVEL = 1",
    "PROF() = floor((LEVEL - 1) / 4) + 2",
    "EXPT() = PROF() * 2",
    "WEALTH_CP = 0",
    "spend_cp(cp) = WEALTH_CP = WEALTH_CP - cp",
    "gain_cp(cp) = spend_cp(-cp)",
    "spend_sp(sp) = spend_cp(sp * 10) / 10",
    "gain_sp(cp) = spend_sp(-cp)",
    "spend_ep(ep) = spend_cp(ep * 50) / 50",
    "gain_ep(ep) = spend_ep(-ep)",
    "spend_gp(gp) = spend_cp(gp * 100) / 100",
    "gain_gp(gp) = spend_gp(-gp)",
    "spend_pp(pp) = spend_cp(pp * 1000) / 1000",
    "gain_pp(pp) = spend_pp(-pp)",
    "avg(roll) = quantity(roll) * (dice(roll) + 1) / 2",
];

struct Builtin {
    name: &'static str,
    call: &'static dyn Fn(&BuiltinCall) -> Res<Outcome>,
}

struct BuiltinCall<'a> {
    gf: &'a Builtin,
    args: &'a [Value],
}

impl<'a> BuiltinCall<'a> {
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

const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "ceil",
        call: &|gfc| gfc.single_decimal().map(|v| Outcome::nat(v.ceil() as i32)),
    },
    Builtin {
        name: "floor",
        call: &|gfc| gfc.single_decimal().map(|v| Outcome::nat(v.floor() as i32)),
    },
    Builtin {
        name: "quantity",
        call: &|gfc| gfc.single_roll().map(|r| Outcome::nat(r.quantity as i32)),
    },
    Builtin {
        name: "dice",
        call: &|gfc| gfc.single_roll().map(|r| Outcome::nat(r.die as i32)),
    },
];

pub fn call(name: &str, args: &[Value]) -> Res<Outcome> {
    for gf in BUILTINS {
        if gf.name == name {
            let gfc = BuiltinCall { gf, args };
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
