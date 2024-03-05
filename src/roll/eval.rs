use std::{collections::HashMap, convert::TryInto, fmt::format};

use super::{
    ast::{Ast, Node},
    Outcome, Roll, RollOutcome,
};

type EvalResult<T> = Result<T, String>;

fn err<T, S: ToString>(msg: S) -> EvalResult<T> {
    Err(msg.to_string())
}

struct Function {
    name: String,
    body: Ast,
    parameters: Vec<String>,
}

pub struct Context {
    variables: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn get(&self, name: &str) -> EvalResult<Value> {
        if let Some(val) = self.variables.get(name) {
            Ok(val.clone())
        } else {
            err(format!("Undefined variable: {name}."))
        }
    }

    fn set<S: ToString>(&mut self, name: S, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    fn call(&self, name: &str, args: &[usize], ast: &Ast) -> EvalResult<Outcome> {
        println!("name({args:?})");
        Ok(Outcome::new(Value::Natural(1)))
    }

    fn define(&mut self, function: Function) {
        self.functions.insert(function.name.clone(), function);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Decimal(f32),
    Natural(u32),
    Outcome(RollOutcome),
    Roll(Roll),
    Values(Vec<u32>),
    Empty,
}

impl Value {
    fn decimal(self) -> EvalResult<f32> {
        match self {
            Self::Decimal(v) => Ok(v),
            Self::Natural(v) => Ok(v as f32),
            Self::Roll(..) => Self::Values(self.values()?).decimal(),
            Self::Outcome(outcome) => Ok(outcome.result as f32),
            Self::Values(_) => Ok(self.natural()? as f32),
            Self::Empty => err("Empty cannot be interpreted as decimal."),
        }
    }

    fn natural(self) -> EvalResult<u32> {
        match self {
            Self::Decimal(v) => Ok(v as u32),
            Self::Natural(v) => Ok(v),
            Self::Outcome(outcome) => Ok(outcome.result),
            Self::Roll(_) => Ok(self.outcome()?.result),
            Self::Values(values) => Ok(values.iter().sum()),
            Self::Empty => err("Empty cannot be interpreted as natural."),
        }
    }

    fn values(self) -> EvalResult<Vec<u32>> {
        match self {
            Self::Decimal(_) => err("Decimal value cannot be interpreted as values."),
            Self::Natural(n) => Ok(vec![n]),
            Self::Roll(..) => Ok(self.outcome()?.values),
            Self::Outcome(outcome) => Ok(outcome.values),
            Self::Values(values) => Ok(values),
            Self::Empty => err("Empty cannot be interpreted as values."),
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
            values.iter().sum()
        };

        Ok(RollOutcome {
            roll,
            values,
            result,
        })
    }
}

impl Outcome {
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
            let (this, outcome) = self.outcome()?;
            Ok((this, outcome.result))
        } else {
            let value = self.value.clone().natural()?;
            Ok((self, value))
        }
    }

    fn decimal(self) -> EvalResult<(Self, f32)> {
        if matches!(self.value, Value::Roll(_) | Value::Outcome(_)) {
            let (this, outcome) = self.outcome()?;
            Ok((this, outcome.result as f32))
        } else {
            let value = self.value.clone().decimal()?;
            Ok((self, value))
        }
    }

    fn arithmetic<F: Fn(f32, f32) -> f32>(self, other: Outcome, f: F) -> EvalResult<Outcome> {
        let (mut this, lhs) = self.decimal()?;
        let (mut that, rhs) = other.decimal()?;
        this.rolls.append(&mut that.rolls);
        Ok(Outcome {
            value: Value::Decimal(f(lhs, rhs)),
            rolls: this.rolls,
        })
    }

    fn add(self, other: Outcome) -> EvalResult<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs + rhs)
    }

    fn sub(self, other: Outcome) -> EvalResult<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs - rhs)
    }

    fn mul(self, other: Outcome) -> EvalResult<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs * rhs)
    }

    fn div(self, other: Outcome) -> EvalResult<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs / rhs)
    }

    fn exp(self, other: Outcome) -> EvalResult<Outcome> {
        self.arithmetic(other, |lhs, rhs| lhs.powf(rhs))
    }

    fn neg(self) -> EvalResult<Outcome> {
        let (this, value) = self.decimal()?;
        Ok(Self {
            value: Value::Decimal(-value),
            rolls: this.rolls,
        })
    }

    fn adv(self) -> EvalResult<Outcome> {
        let mut roll = self.value.roll()?;
        roll.advantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    fn disadv(self) -> EvalResult<Self> {
        let mut roll = self.value.roll()?;
        roll.disadvantage = true;
        Ok(Self {
            value: Value::Roll(roll),
            rolls: self.rolls,
        })
    }

    fn sort(self) -> EvalResult<Self> {
        let (this, mut values) = self.values()?;
        values.sort();
        Ok(Self {
            value: Value::Values(values),
            rolls: this.rolls,
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
) -> EvalResult<Outcome> {
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

    context.define(Function {
        name: name.to_string(),
        body,
        parameters,
    });
    Ok(Outcome::new(Value::Empty))
}

fn assign(
    ast: &Ast,
    context: &mut Context,
    destination: usize,
    definition: usize,
) -> EvalResult<Outcome> {
    match ast.get(destination) {
        Some(Node::Identifier(name)) => {
            let value = evaluate(ast, context, definition)?.value;
            context.set(name, value);
            context.get(name).map(Outcome::new)
        }
        Some(Node::Call(name, args)) => define(ast, context, name, args, definition),
        invalid => err(format!("{invalid:?} is not a valid assignment target.")),
    }
}

fn evaluate(ast: &Ast, context: &mut Context, index: usize) -> EvalResult<Outcome> {
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
            Node::Call(name, args) => context.call(&name, &args, ast),
            Node::Identifier(name) => context.get(&name).map(Outcome::new),
        }
    } else {
        err("Attempted to evaluate expression which did not exist.")
    }
}

pub fn eval(ast: &Ast, context: &mut Context) -> EvalResult<Outcome> {
    evaluate(ast, context, ast.start())
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
        let result = eval(&parse("4d6k3 + 2d4 + d20d + 2d10a")).unwrap();
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
        assert_eq!(eval(&ast).unwrap().rolls.len(), 1);
    }

    #[test]
    fn test_eval() {
        let ast = parse("2 + 3 - 4 * 5");
        assert_eq!(
            eval(&ast).unwrap().value.decimal().unwrap(),
            2.0 + 3.0 - 4.0 * 5.0
        );
    }

    #[test]
    fn test_assignment() {
        let mut context = Context::new();
        let ast = parse("var = 2 + 3 - 1");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(context.get("var").unwrap().natural().unwrap(), 2 + 3 - 1);
    }

    #[test]
    fn test_definition() {
        let mut context = Context::new();
        let ast = parse("func(x, y) := x + y");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(
            context.functions.get("func").unwrap().body.render(),
            "x + y"
        );
    }
}
