use std::{collections::HashMap, fmt::Display, rc::Rc, sync::atomic::AtomicUsize};

use crate::{
    builtins::{self, DEFAULT_GLOBALS},
    err,
    outcome::Outcome,
    parse, Res,
};

use super::{
    ast::{Ast, Node},
    value::Value,
};

struct Function {
    name: String,
    body: Ast,
    parameters: Vec<String>,

    /// Unique ID of this function. This keeps track of declaration order, which
    /// is important because when we are saving defined functions, we need to
    /// ensure that all functions used within a function are available in the
    /// scope the function is evaluated in.
    id: usize,
}

impl Function {
    fn new<S: ToString>(name: S, body: Ast, parameters: Vec<String>) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        Self {
            name: name.to_string(),
            body,
            parameters,
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}) = {}",
            &self.name,
            self.parameters.join(", "),
            self.body.render()
        )
    }
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
    initialised: bool,
}

impl Context {
    const GLOBAL_SCOPE_INDEX: usize = 0;
    const USER_SCOPE_INDEX: usize = 1;

    pub fn new() -> Self {
        let mut context = Self {
            scopes: vec![Scope::new(), Scope::new()],
            initialised: false,
        };

        // Add default globals by evaluating them.
        for definition in DEFAULT_GLOBALS {
            if let Err(e) = context.eval(&definition) {
                panic!(
                    "Failed to evaluate global: {} Definition: {}",
                    e, definition
                );
            };
        }

        context.initialised = true;
        context
    }

    fn definition_scope(&mut self) -> &mut Scope {
        let index = if self.initialised {
            Self::USER_SCOPE_INDEX
        } else {
            Self::GLOBAL_SCOPE_INDEX
        };
        self.scopes.get_mut(index).expect("Permanent scope popped.")
    }

    fn push_scope(&mut self) -> &mut Scope {
        self.scopes.push(Scope::new());
        self.scopes.last_mut().unwrap()
    }

    fn pop_scope(&mut self) {
        // Never pop user or global scope.
        if self.scopes.len() > 2 {
            self.scopes.pop();
        }
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
        self.definition_scope()
            .variables
            .insert(name.to_string(), value);
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
        self.definition_scope()
            .functions
            .insert(function.name.clone(), Rc::new(function));
    }

    fn call(&mut self, name: &str, args: Vec<Value>) -> Res<Outcome> {
        if let Some(function) = self.get_function(name) {
            let scope = self.push_scope();
            check_argument_count(name, function.parameters.len(), &args)?;
            for (name, value) in function.parameters.iter().zip(args) {
                scope.variables.insert(name.clone(), value);
            }
            let ret = evaluate(&function.body, self, function.body.start());
            self.pop_scope();
            ret
        } else {
            builtins::call(name, &args)
        }
    }

    pub fn eval(&mut self, statement: &str) -> Res<()> {
        let ast = parse(statement)?;
        evaluate(&ast, self, ast.start()).map(|_| ())
    }

    pub fn dump_to_string(&self) -> String {
        let mut ret = String::new();

        let user_scope = self
            .scopes
            .get(Self::USER_SCOPE_INDEX)
            .expect("User scope popped.");

        // Sort functions by definition order.
        let mut functions: Vec<&Rc<Function>> = user_scope.functions.values().collect();
        functions.sort_by(|a, b| (a.id).cmp(&b.id));
        for func in functions {
            ret += &format!("{func}\n");
        }

        for (k, v) in &user_scope.variables {
            ret += &format!("{k} = {v}\n");
        }
        ret
    }
}

pub fn check_argument_count(name: &str, count: usize, args: &[Value]) -> Res<()> {
    if count != args.len() {
        err(format!(
            "Incorrect number of arguments: {name} expects {count}."
        ))
    } else {
        Ok(())
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

    context.define_function(Function::new(name, body, parameters));
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
            Node::Value(val) => Ok(Outcome::new(val.clone())),
            Node::Call(name, args) => call(ast, context, name, args),
            Node::Identifier(name) => variable(ast, context, name),
        }
    } else {
        err("Attempted to evaluate expression which did not exist.")
    }
}

pub fn eval_roll(ast: &Ast, context: &mut Context) -> Res<Outcome> {
    let outcome = evaluate(ast, context, ast.start())?;
    if matches!(outcome.value, Value::Roll(_)) {
        outcome.natural().map(|oc| oc.0)
    } else {
        Ok(outcome)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parser::parse,
        roll::{Roll, RollOutcome},
        token::tokenise,
    };

    use super::*;

    fn ast_of(input: &str) -> Ast {
        parse(&tokenise(input).unwrap()).unwrap()
    }

    fn eval_value(ast: Ast) -> Value {
        evaluate(&ast, &mut Context::new(), ast.start())
            .unwrap()
            .value
    }

    #[test]
    fn test_natural() {
        assert_eq!(eval_value(ast_of("16")), Value::Natural(16));
    }

    #[test]
    fn test_roll() {
        assert_eq!(
            eval_value(ast_of("4d12")),
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
            eval_value(ast_of("5 + 4 + 3 + 2 + 1")).natural().unwrap(),
            5 + 4 + 3 + 2 + 1
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            eval_value(ast_of("5 * 4 ^ 2 / 3 + 2 - 1"))
                .decimal()
                .unwrap(),
            5.0 * 4.0_f32.powf(2.0) / 3.0 + 2.0 - 1.0
        );
    }

    #[test]
    fn test_rolls() {
        let result = eval_roll(&ast_of("4d6k3 + 2d4 + d20d + 2d10a"), &mut Context::new()).unwrap();
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
        let ast = ast_of("8d8s");
        assert_eq!(eval_roll(&ast, &mut Context::new()).unwrap().rolls.len(), 1);
    }

    #[test]
    fn test_eval() {
        let ast = ast_of("2 + 3 - 4 * 5");
        assert_eq!(
            eval_roll(&ast, &mut Context::new())
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
        let ast = ast_of("var = 2 + 3 - 1");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(
            context.get_variable("var").unwrap().natural().unwrap(),
            2 + 3 - 1
        );
    }

    #[test]
    fn test_definition() {
        let mut context = Context::new();
        let ast = ast_of("func(x, y) := x + y");
        evaluate(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(context.get_function("func").unwrap().body.render(), "x + y");
    }
}
