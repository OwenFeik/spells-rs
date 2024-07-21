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
    "gain_sp(sp) = spend_sp(-sp)",
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
    args: usize,
    func: &'static dyn Fn(BuiltinCall) -> Res<Outcome>,
}

impl Builtin {
    fn call(&self, gfc: BuiltinCall) -> Res<Outcome> {
        eval::check_argument_count(self.name, self.args, &gfc.args)?;
        (self.func)(gfc)
    }
}

struct BuiltinCall<'a> {
    gf: &'a Builtin,
    args: Vec<Value>,
}

impl<'a> BuiltinCall<'a> {
    fn pop(&mut self) -> Res<Value> {
        if let Some(val) = self.args.pop() {
            Ok(val)
        } else {
            Err(format!(
                "Incorrect number of arguments: {} expects {}.",
                self.gf.name, self.gf.args
            ))
        }
    }

    fn pop_decimal(&mut self) -> Res<f64> {
        self.pop().and_then(Value::decimal)
    }

    fn pop_roll(&mut self) -> Res<Roll> {
        self.pop().and_then(Value::roll)
    }

    fn pop_list(&mut self) -> Res<Vec<Value>> {
        self.pop().and_then(Value::list)
    }

    fn pop_natural(&mut self) -> Res<i64> {
        self.pop().and_then(Value::natural)
    }

    fn pop_string(&mut self) -> Res<String> {
        self.pop().and_then(Value::string)
    }
}

const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "ceil",
        args: 1,
        func: &|mut gfc| gfc.pop_decimal().map(|v| Outcome::nat(v.ceil() as i64)),
    },
    Builtin {
        name: "floor",
        args: 1,
        func: &|mut gfc| gfc.pop_decimal().map(|v| Outcome::nat(v.floor() as i64)),
    },
    Builtin {
        name: "quantity",
        args: 1,
        func: &|mut gfc| gfc.pop_roll().map(|r| Outcome::nat(r.quantity as i64)),
    },
    Builtin {
        name: "get",
        args: 2,
        func: &|mut gfc| {
            let index = gfc.pop_natural()?;
            let list = gfc.pop_list()?;

            if index < 0 || index as usize >= list.len() {
                Err(format!(
                    "Index {index} of range for list of length {}.",
                    list.len()
                ))
            } else {
                Ok(Outcome::new(list.get(index as usize).cloned().unwrap()))
            }
        },
    },
    Builtin {
        name: "dice",
        args: 1,
        func: &|mut gfc| gfc.pop_roll().map(|r| Outcome::nat(r.die as i64)),
    },
    Builtin {
        name: "print",
        args: 1,
        func: &|mut gfc| {
            gfc.pop_string().map(|s| {
                println!("{s}");
                Outcome::new(Value::Empty)
            })
        },
    },
];

pub fn call(name: &str, args: Vec<Value>) -> Res<Outcome> {
    for gf in BUILTINS {
        if gf.name == name {
            return gf.call(BuiltinCall { gf, args });
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
            call("ceil", vec![Value::Decimal(2.5)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            3.0
        );
        assert_eq!(
            call("ceil", vec![Value::Decimal(2.2)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            3.0
        );
        assert_eq!(
            call("ceil", vec![Value::Decimal(-2.2)])
                .and_then(Outcome::decimal)
                .map(|tup| tup.1)
                .unwrap(),
            -2.0
        );
        assert!(call("ceil", vec![Value::Empty]).is_err());
    }

    #[test]
    fn test_roll() {
        assert_eq!(
            call("dice", vec![Value::Roll(Roll::new(8, 8))])
                .and_then(Outcome::natural)
                .map(|tup| tup.1)
                .unwrap(),
            8
        );
    }
}
