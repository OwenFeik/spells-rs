use crate::{context::Context, err, operator::Operator, outcome::Outcome, Res};

use super::{
    ast::{Ast, Node},
    value::Value,
};

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

    context.define_function(name, body, parameters);
    Ok(Outcome::empty())
}

fn assign(ast: &Ast, context: &mut Context, destination: usize, definition: usize) -> Res<Outcome> {
    match ast.get(destination) {
        Some(Node::Identifier(name)) => {
            let value = evaluate_node(ast, context, definition)?.value;
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
        arg_values.push(evaluate_node(ast, context, *arg)?.value);
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

fn list(ast: &Ast, context: &mut Context, values: &[usize]) -> Res<Outcome> {
    let mut list = Vec::new();
    for &index in values {
        let val = evaluate_node(ast, context, index)?;
        list.push(val.value);
    }
    Ok(Outcome::new(Value::List(list)))
}

fn binary(ast: &Ast, context: &mut Context, op: Operator, lhs: usize, rhs: usize) -> Res<Outcome> {
    if matches!(op, Operator::Assign) {
        assign(ast, context, lhs, rhs)
    } else {
        let lhs_val = evaluate_node(ast, context, lhs)?;
        let rhs_val = evaluate_node(ast, context, rhs)?;
        match op {
            Operator::Assign => err("Operator::Assign doesn't match Operator::Assign."),
            Operator::Discard => Ok(rhs_val),
            Operator::And => lhs_val.and(rhs_val),
            Operator::Or => lhs_val.or(rhs_val),
            Operator::Add => lhs_val.add(rhs_val),
            Operator::Sub => lhs_val.sub(rhs_val),
            Operator::Mul => lhs_val.mul(rhs_val),
            Operator::Div => lhs_val.div(rhs_val),
            Operator::Exp => lhs_val.exp(rhs_val),
            Operator::Keep => lhs_val.keep(rhs_val),
            Operator::Equal => lhs_val.equal(rhs_val),
            Operator::GreaterThan => lhs_val.greater_than(rhs_val),
            Operator::LessThan => lhs_val.less_than(rhs_val),
            Operator::GreaterEqual => lhs_val.greater_equal(rhs_val),
            Operator::LessEqual => lhs_val.less_equal(rhs_val),
            Operator::Sentinel
            | Operator::Not
            | Operator::Neg
            | Operator::Adv
            | Operator::DisAdv
            | Operator::Sort => Err(format!("Not a binary operator: {}", op.str())),
        }
    }
}

fn unary(ast: &Ast, context: &mut Context, op: Operator, arg: usize) -> Res<Outcome> {
    let val = evaluate_node(ast, context, arg)?;
    match op {
        Operator::Not => val.not(),
        Operator::Neg => val.neg(),
        Operator::Adv => val.adv(),
        Operator::DisAdv => val.disadv(),
        Operator::Sort => val.sort(),
        _ => Err(format!("Not a unary operator: {}", op.str())),
    }
}

fn condition(
    ast: &Ast,
    context: &mut Context,
    cond: usize,
    block: usize,
    fail: Option<usize>,
) -> Res<Outcome> {
    let condition = evaluate_node(ast, context, cond)?.value.bool()?;
    if condition {
        evaluate_node(ast, context, block)
    } else if let Some(node) = fail {
        evaluate_node(ast, context, node)
    } else {
        Ok(Outcome::new(Value::Empty))
    }
}

fn evaluate_node(ast: &Ast, context: &mut Context, index: usize) -> Res<Outcome> {
    if let Some(expr) = ast.get(index) {
        match expr {
            Node::Value(val) => Ok(Outcome::new(val.clone())),
            Node::Identifier(name) => variable(ast, context, name),
            Node::List(values) => list(ast, context, values),
            &Node::Binary(lhs, op, rhs) => binary(ast, context, op, lhs, rhs),
            &Node::Unary(arg, op) => unary(ast, context, op, arg),
            Node::Call(name, args) => call(ast, context, name, args),
            &Node::If(cond, expr, fail) => condition(ast, context, cond, expr, fail),
        }
    } else {
        err("Attempted to evaluate expression which did not exist.")
    }
}

pub fn evaluate(ast: &Ast, context: &mut Context) -> Res<Outcome> {
    if ast.is_empty() {
        Ok(Outcome::empty())
    } else {
        evaluate_node(ast, context, ast.start())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        context::Context,
        eval,
        parser::parse,
        roll::{Roll, RollOutcome},
        token::tokenise,
    };

    use super::*;

    fn ast_of(input: &str) -> Ast {
        parse(&tokenise(input).unwrap()).unwrap()
    }

    fn eval_value(ast: Ast) -> Value {
        evaluate_node(&ast, &mut Context::empty(), ast.start())
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
            5.0 * 4.0_f64.powf(2.0) / 3.0 + 2.0 - 1.0
        );
    }

    #[test]
    fn test_rolls() {
        let result = evaluate(&ast_of("4d6k3 + 2d4 + d20d + 2d10a"), &mut Context::empty())
            .unwrap()
            .resolved()
            .unwrap();
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
                rolls: vec![1, 2, 3, 4, 5, 6, 7, 8],
                result: 36,
            }),
            rolls: Vec::new(),
        };
        let values = expr.keep(Outcome::nat(6)).unwrap().value.rolls().unwrap();
        assert_eq!(values, vec![3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_sort() {
        let expr = Outcome {
            value: Value::Rolls(vec![3, 4, 1, 7]),
            rolls: Vec::new(),
        };
        let values = expr.sort().unwrap().value.rolls().unwrap();
        assert_eq!(values, vec![1, 3, 4, 7]);
    }

    #[test]
    fn test_sort_outcomes() {
        let ast = ast_of("8d8s");
        assert_eq!(
            evaluate(&ast, &mut Context::empty()).unwrap().rolls.len(),
            1
        );
    }

    #[test]
    fn test_eval() {
        let ast = ast_of("2 + 3 - 4 * 5");
        assert_eq!(
            evaluate(&ast, &mut Context::empty())
                .unwrap()
                .value
                .decimal()
                .unwrap(),
            2.0 + 3.0 - 4.0 * 5.0
        );
    }

    #[test]
    fn test_assignment() {
        let mut context = Context::empty();
        let ast = ast_of("var = 2 + 3 - 1");
        evaluate_node(&ast, &mut context, ast.start()).unwrap();
        assert_eq!(
            context.get_variable("var").unwrap().natural().unwrap(),
            2 + 3 - 1
        );
    }

    #[test]
    fn test_join_strings() {
        assert_eq!(
            eval(r#""abc" + "def""#, &mut Context::empty())
                .unwrap()
                .to_string(),
            r#""abcdef""#
        )
    }

    #[test]
    fn test_discard() {
        assert_eq!(
            eval("1; 2", &mut Context::empty()).unwrap(),
            Outcome::nat(2)
        )
    }

    #[test]
    fn test_discard_assignment() {
        let context = &mut Context::empty();
        assert_eq!(eval("a = 2; b = 3", context).unwrap(), Outcome::nat(3));
        assert_eq!(context.get_variable("a"), Some(Value::Natural(2)));
        assert_eq!(context.get_variable("b"), Some(Value::Natural(3)));
    }

    #[test]
    fn test_multiple_statement_function() {
        let context = &mut Context::empty();
        assert_eq!(eval("a = 1; b = 2", context).unwrap(), Outcome::nat(2));
        eval("incr() = a = a + 1; b = b + 1", context).unwrap();
        eval("incr()", context).unwrap();
        assert_eq!(context.get_variable("a").unwrap().natural().unwrap(), 2);
        assert_eq!(context.get_variable("b").unwrap().natural().unwrap(), 3);
    }

    #[test]
    fn test_multiline_statement() {
        let mut context = &mut Context::empty();
        eval(
            r#"
            if true then
                a = 4
            "#,
            &mut context,
        )
        .unwrap();
        assert_eq!(context.get_variable("a").unwrap(), Value::Natural(4));
    }
}
