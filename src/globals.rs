use crate::{err, outcome::Outcome, value::Value, Res};

struct GlobalFunction {
    name: &'static str,
    call: &'static dyn Fn(&GlobalFunctionCall) -> Res<Outcome>,
}

struct GlobalFunctionCall<'a> {
    gf: &'a GlobalFunction,
    args: &'a [Value],
}

impl<'a> GlobalFunctionCall<'a> {
    fn single_decimal(&self) -> Res<f32> {
        if let Some(value) = self.args.first() {
            value.clone().decimal()
        } else {
            err(format!("{} expects a single argument.", self.gf.name))
        }
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
