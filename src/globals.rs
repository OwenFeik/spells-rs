use crate::{err, eval, outcome::Outcome, value::Value, Res};

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
        eval::check_argument_count(self.gf.name, 1, self.args)?;

        // Index safe because we checked length is 1.
        self.args[0].clone().decimal()
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
