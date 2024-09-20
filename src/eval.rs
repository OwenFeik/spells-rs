use crate::{context::Context, err, operator::Operator, outcome::Outcome, Res};

use super::{
    ast::{Ast, Node},
    value::Value,
};

struct EvalCtx<'a> {
    ast: &'a Ast,
    context: &'a mut Context,
    scope: usize,
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

fn define_func(ctx: &mut EvalCtx, name: &str, args: &[usize], definition: usize) -> Res<Outcome> {
    let mut parameters = Vec::new();
    for &arg in args {
        let Some(Node::Identifier(name)) = ctx.ast.get(arg) else {
            return err(format!(
                "Invalid argument signature: {:?}.",
                ctx.ast.get(arg)
            ));
        };
        parameters.push(name.clone());
    }

    let Some(body) = ctx.ast.subtree(definition) else {
        return err("Failed to get subtree for definition.");
    };

    ctx.context
        .define_function(ctx.scope, name, body, parameters);
    Ok(Outcome::empty())
}

fn assign(ctx: &mut EvalCtx, destination: usize, definition: usize) -> Res<Outcome> {
    match ctx.ast.get(destination) {
        Some(Node::Identifier(name)) => {
            let value = evaluate_node(ctx, definition)?.value;
            ctx.context.set_variable(ctx.scope, name, value.clone());
            Ok(Outcome::new(value))
        }
        invalid => err(format!("{invalid:?} is not a valid assignment target.")),
    }
}

fn define(ctx: &mut EvalCtx, signature: usize, definition: usize) -> Res<Outcome> {
    match ctx.ast.get(signature) {
        Some(Node::Call(name, args)) => define_func(ctx, name, args, definition),
        invalid => err(format!("{invalid:?} is not a valid function signature.")),
    }
}

fn call(ctx: &mut EvalCtx, name: &str, args: &[usize]) -> Res<Outcome> {
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(evaluate_node(ctx, *arg)?.value);
    }
    ctx.context.call(ctx.scope, name, arg_values)
}

/// Attempts to return the value of the given name in the current context. If
/// not found attempts to call a function with the given name with no
/// parameters.
fn variable(ctx: &mut EvalCtx, name: &str) -> Res<Outcome> {
    if let Some(value) = ctx.context.get_variable(ctx.scope, name) {
        return Ok(Outcome::new(value.clone()));
    } else {
        let call_res = call(ctx, name, &[]);
        if call_res.is_ok() {
            return call_res;
        }
    }
    err(format!("Undefined variable: {name}."))
}

fn list(ctx: &mut EvalCtx, values: &[usize]) -> Res<Outcome> {
    let mut list = Vec::new();
    for &index in values {
        let val = evaluate_node(ctx, index)?;
        list.push(val.value);
    }
    Ok(Outcome::new(Value::List(list)))
}

fn binary(ctx: &mut EvalCtx, op: Operator, lhs: usize, rhs: usize) -> Res<Outcome> {
    if matches!(op, Operator::Assign) {
        assign(ctx, lhs, rhs)
    } else if matches!(op, Operator::Define) {
        define(ctx, lhs, rhs)
    } else {
        let lhs_val = evaluate_node(ctx, lhs)?;
        let rhs_val = evaluate_node(ctx, rhs)?;
        match op {
            Operator::Assign => err("Operator::Assign doesn't match Operator::Assign."),
            Operator::Define => err("Operator::Define doesn't match Operator::Define."),
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
            | Operator::DisAdv => Err(format!("Not a binary operator: {}", op.str())),
        }
    }
}

fn unary(ctx: &mut EvalCtx, op: Operator, arg: usize) -> Res<Outcome> {
    let val = evaluate_node(ctx, arg)?;
    match op {
        Operator::Not => val.not(),
        Operator::Neg => val.neg(),
        Operator::Adv => val.adv(),
        Operator::DisAdv => val.disadv(),
        _ => Err(format!("Not a unary operator: {}", op.str())),
    }
}

fn condition(ctx: &mut EvalCtx, cond: usize, block: usize, fail: Option<usize>) -> Res<Outcome> {
    let condition = evaluate_node(ctx, cond)?.value.bool()?;
    if condition {
        evaluate_node(ctx, block)
    } else if let Some(node) = fail {
        evaluate_node(ctx, node)
    } else {
        Ok(Outcome::new(Value::Empty))
    }
}

fn evaluate_node(ctx: &mut EvalCtx, index: usize) -> Res<Outcome> {
    if let Some(expr) = ctx.ast.get(index) {
        match expr {
            Node::Value(val) => Ok(Outcome::new(val.clone())),
            Node::Identifier(name) => variable(ctx, name),
            Node::List(values) => list(ctx, values),
            &Node::Binary(lhs, op, rhs) => binary(ctx, op, lhs, rhs),
            &Node::Unary(arg, op) => unary(ctx, op, arg),
            Node::Call(name, args) => call(ctx, name, args),
            &Node::If(cond, expr, fail) => condition(ctx, cond, expr, fail),
        }
    } else {
        err("Attempted to evaluate expression which did not exist.")
    }
}

pub fn evaluate(ast: &Ast, context: &mut Context, scope: usize) -> Res<Outcome> {
    if ast.is_empty() {
        Ok(Outcome::empty())
    } else {
        let ctx = &mut EvalCtx {
            ast,
            context,
            scope,
        };
        evaluate_node(ctx, ast.start())
    }
}

pub fn evaluate_tome(statements: &[Ast], context: &mut Context, scope: usize) -> Res<()> {
    for statement in statements {
        evaluate_node(
            &mut EvalCtx {
                ast: statement,
                context,
                scope,
            },
            statement.start(),
        )?;
    }
    Ok(())
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
        evaluate(&ast, &mut Context::empty(), Context::GLOBAL_SCOPE)
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
        let result = evaluate(
            &ast_of("4d6k3 + 2d4 + d20d + 2d10a"),
            &mut Context::empty(),
            Context::GLOBAL_SCOPE,
        )
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
    fn test_eval() {
        assert_eq!(
            eval_value(ast_of("2 + 3 - 4 * 5")).decimal().unwrap(),
            2.0 + 3.0 - 4.0 * 5.0
        );
    }

    #[test]
    fn test_assignment() {
        let mut context = Context::empty();
        let ast = ast_of("var = 2 + 3 - 1");
        evaluate(&ast, &mut context, Context::GLOBAL_SCOPE).unwrap();
        assert_eq!(
            context
                .get_variable(Context::GLOBAL_SCOPE, "var")
                .cloned()
                .unwrap()
                .natural()
                .unwrap(),
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
        assert_eq!(
            context.get_variable(Context::GLOBAL_SCOPE, "a").cloned(),
            Some(Value::Natural(2))
        );
        assert_eq!(
            context.get_variable(Context::GLOBAL_SCOPE, "b").cloned(),
            Some(Value::Natural(3))
        );
    }

    #[test]
    fn test_multiple_statement_function() {
        let context = &mut Context::empty();
        assert_eq!(eval("a = 1; b = 2", context).unwrap(), Outcome::nat(2));
        dbg!(&context);
        eval("incr() := a = a + 1; b = b + 1", context).unwrap();
        dbg!(&context);
        eval("incr()", context).unwrap();
        dbg!(&context);
        assert_eq!(
            context
                .get_variable(Context::GLOBAL_SCOPE, "a")
                .cloned()
                .unwrap()
                .natural()
                .unwrap(),
            2
        );
        assert_eq!(
            context
                .get_variable(Context::GLOBAL_SCOPE, "b")
                .cloned()
                .unwrap()
                .natural()
                .unwrap(),
            3
        );
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
        assert_eq!(
            context
                .get_variable(Context::GLOBAL_SCOPE, "a")
                .cloned()
                .unwrap(),
            Value::Natural(4)
        );
    }
}
