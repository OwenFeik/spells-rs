use std::{collections::HashMap, rc::Rc};

use crate::{err, Res};

use super::{
    ast::{Ast, Node},
    value::Value,
    Outcome, Roll, RollOutcome,
};

struct Function {
    name: String,
    body: Ast,
    parameters: Vec<String>,
}

struct Scope {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Rc<Function>>,
}

impl Scope {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}

pub struct Context {
    scopes: Vec<Scope>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    fn globals(&mut self) -> &mut Scope {
        self.scopes.get_mut(0).expect("Global scope popped.")
    }

    fn get_variable(&self, name: &str) -> Option<Value> {
        self.scopes
            .iter()
            .rev()
            .map(|scope| scope.variables.get(name).cloned())
            .find(Option::is_some)
            .flatten()
    }

    fn set_variable<S: ToString>(&mut self, name: S, value: Value) {
        self.globals().variables.insert(name.to_string(), value);
    }

    fn get_function(&self, name: &str) -> Option<Rc<Function>> {
        self.scopes
            .iter()
            .rev()
            .map(|scope| scope.functions.get(name).cloned())
            .find(Option::is_some)
            .flatten()
    }

    fn define_function(&mut self, function: Function) {
        self.globals()
            .functions
            .insert(function.name.clone(), Rc::new(function));
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Res<Outcome> {
        if let Some(function) = self.get_function(name) {
            let mut scope = Scope::new();
            if function.parameters.len() != args.len() {
                return err(format!(
                    "Incorrect number of arguments. {name} expects {} arguments.",
                    function.parameters.len()
                ));
            }
            for (name, value) in function.parameters.iter().zip(args) {
                scope.variables.insert(name.clone(), value);
            }
            self.scopes.push(scope);
            let ret = evaluate(&function.body, self, function.body.start());
            self.scopes.pop();
            ret
        } else {
            err(format!("Undefined function: {name}."))
        }
    }
}

impl Outcome {
    fn new(value: Value) -> Self {
        Self {
            value,
            rolls: Vec::new(),
        }
    }

    fn outcome(mut self) -> Res<(Self, RollOutcome)> {
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

    fn values(self) -> Res<(Self, Vec<u32>)> {
        if matches!(self.value, Value::Roll(_)) {
            let (val, outcome) = self.outcome()?;
            Ok((val, Value::Outcome(outcome).values()?))
        } else {
            let values = self.value.clone().values()?;
            Ok((self, values))
        }
    }

    fn natural(self) -> Res<(Self, u32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, outcome) = self.outcome()?;
            Ok((this, outcome.result))
        } else {
            let value = self.value.clone().natural()?;
            Ok((self, value))
        }
    }

    fn decimal(self) -> Res<(Self, f32)> {
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

    fn add(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs + rhs)
    }

    fn sub(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs - rhs)
    }

    fn mul(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs * rhs)
    }

    fn div(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs / rhs)
    }

    fn exp(self, other: Outcome) -> Res<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs.powf(rhs))
    }

    fn neg(self) -> Res<Outcome> {
        let (this, value) = self.decimal()?;
        Ok(Self {
            value: Value::Decimal(-value),
            rolls: this.rolls,
        })
    }

    fn adv(self) -> Res<Outcome> {
        let mut roll = self.value.roll()?;
        roll.advantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    fn disadv(self) -> Res<Self> {
        let mut roll = self.value.roll()?;
        roll.disadvantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    fn sort(self) -> Res<Self> {
        let (this, mut values) = self.values()?;
        values.sort();
        Ok(Self {
            value: Value::Values(values),
            rolls: this.rolls,
        })
    }

    fn keep(self, rhs: Self) -> Res<Self> {
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

fn define(
    ast: &Ast,
    context: &mut Context,
    name: &str,
    args: &[usize],
    definition: usize,
) -> Res<Outcome> {
    let mut parameters = Vec::new();
    for &arg in args {
        let Some(Node::Identifier(name)) = ast.get(arg) else {
            return err(format!("Invalid argument signature: {:?}.", ast.get(arg)));
        };
        parameters.push(name.clone());
    }

    let Some(body) = ast.subtree(definition) else {
        return err("Failed to get subtree for definition.");
    };

    context.define_function(Function {
        name: name.to_string(),
        body,
        parameters,
    });
    Ok(Outcome::new(Value::Empty))
}

fn assign(ast: &Ast, context: &mut Context, destination: usize, definition: usize) -> Res<Outcome> {
    match ast.get(destination) {
        Some(Node::Identifier(name)) => {
            let value = evaluate(ast, context, definition)?.value;
            context.set_variable(name, value.clone());
            Ok(Outcome::new(value))
        }
        Some(Node::Call(name, args)) => define(ast, context, name, args, definition),
        invalid => err(format!("{invalid:?} is not a valid assignment target.")),
    }
}

fn call(ast: &Ast, context: &mut Context, name: &str, args: &[usize]) -> Res<Outcome> {
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(evaluate(ast, context, *arg)?.value);
    }
    context.call(name, arg_values)
}

/// Attempts to return the value of the given name in the current context. If
/// not found attempts to call a function with the given name with no
/// parameters.
fn variable(ast: &Ast, context: &mut Context, name: &str) -> Res<Outcome> {
    if let Some(value) = context.get_variable(name) {
        return Ok(Outcome::new(value));
    } else {
        let call_res = call(ast, context, name, &[]);
        if call_res.is_ok() {
            return call_res;
        }
    }
    err(format!("Undefined variable: {name}."))
}

fn evaluate(ast: &Ast, context: &mut Context, index: usize) -> Res<Outcome> {
    if let Some(expr) = ast.get(index) {
        match expr {
            &Node::Assign(destination, definition) => assign(ast, context, destination, definition),
            &Node::Add(lhs, rhs) => evaluate(ast, context, lhs)?.add(evaluate(ast, context, rhs)?),
            &Node::Sub(lhs, rhs) => evaluate(ast, context, lhs)?.sub(evaluate(ast, context, rhs)?),
            &Node::Mul(lhs, rhs) => evaluate(ast, context, lhs)?.mul(evaluate(ast, context, rhs)?),
            &Node::Div(lhs, rhs) => evaluate(ast, context, lhs)?.div(evaluate(ast, context, rhs)?),
            &Node::Exp(lhs, rhs) => evaluate(ast, context, lhs)?.exp(evaluate(ast, context, rhs)?),
            &Node::Neg(arg) => evaluate(ast, context, arg)?.neg(),
            &Node::Adv(arg) => evaluate(ast, context, arg)?.adv(),
            &Node::DisAdv(arg) => evaluate(ast, context, arg)?.disadv(),
            &Node::Sort(arg) => evaluate(ast, context, arg)?.sort(),
            &Node::Keep(lhs, rhs) => {
                evaluate(ast, context, lhs)?.keep(evaluate(ast, context, rhs)?)
            }
            &Node::Roll(q, d) => Ok(Outcome::roll(q, d)),
            &Node::Natural(v) => Ok(Outcome::nat(v)),
            Node::Call(name, args) => call(ast, context, name, args),
            Node::Identifier(name) => variable(ast, context, name),
        }
    } else {
        err("Attempted to evaluate expression which did not exist.")
    }
}

pub fn eval(ast: &Ast, context: &mut Context) -> Res<Outcome> {
    let outcome = evaluate(ast, context, ast.start())?;
    if matches!(outcome.value, Value::Roll(_)) {
        outcome.natural().map(|oc| oc.0)
    } else {
        Ok(outcome)
    }
}

#[cfg(test)]
mod test {
    use crate::roll::{ast::lex, token::tokenise};

    use super::*;

    fn parse(input: &str) -> Ast {
        lex(&tokenise(input).unwrap()).unwrap()
    }

    fn eval_value(ast: Ast) -> Value {
        evaluate(&ast, &mut Context::new(), ast.start())
            .unwrap()
            .value
    }

    #[test]
    fn test_natural() {
        assert_eq!(eval_value(parse("16")), Value::Natural(16));
    }

    #[test]
    fn test_roll() {
        assert_eq!(
            eval_value(parse("4d12")),
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
            eval_value(parse("5 + 4 + 3 + 2 + 1")).natural().unwrap(),
            5 + 4 + 3 + 2 + 1
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            eval_value(parse("5 * 4 ^ 2 / 3 + 2 - 1"))
                .decimal()
                .unwrap(),
            5.0 * 4.0_f32.powf(2.0) / 3.0 + 2.0 - 1.0
        );
    }

    #[test]
    fn test_rolls() {
        let result = eval(&parse("4d6k3 + 2d4 + d20d + 2d10a"), &mut Context::new()).unwrap();
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
        let expr = Outcome {
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
        let values = expr.keep(Outcome::nat(6)).unwrap().value.values().unwrap();
        assert_eq!(values, vec![3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_sort() {
        let expr = Outcome {
            value: Value::Values(vec![3, 4, 1, 7]),
            rolls: Vec::new(),
        };
        let values = expr.sort().unwrap().value.values().unwrap();
        assert_eq!(values, vec![1, 3, 4, 7]);
    }

    #[test]
    fn test_sort_outcomes() {
        let ast = parse("8d8s");
        assert_eq!(eval(&ast, &mut Context::new()).unwrap().rolls.len(), 1);
    }

    #[test]
    fn test_eval() {
        let ast = parse("2 + 3 - 4 * 5");
        assert_eq!(
            eval(&ast, &mut Context::new())
                .unwrap()
                .value
                .decimal()
                .unwrap(),
            2.0 + 3.0 - 4.0 * 5.0
        );
    }

    #[test]
    fn test_assignment() {
        let mut context = Context::new();
        let ast = parse("var = 2 + 3 - 1");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(
            context.get_variable("var").unwrap().natural().unwrap(),
            2 + 3 - 1
        );
    }

    #[test]
    fn test_definition() {
        let mut context = Context::new();
        let ast = parse("func(x, y) := x + y");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(context.get_function("func").unwrap().body.render(), "x + y");
    }
}
