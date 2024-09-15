use crate::{
    ast::{Ast, Node},
    err,
    operator::Operator,
    roll::Roll,
    token::{Tok, TokenList},
    value::Value,
    Res,
};

use super::token::Token;

struct Parser<'a> {
    source: &'a TokenList,
    input: &'a [Token],
    operators: Vec<Operator>,
    operands: Vec<usize>,
    operators_scopes: Vec<Vec<Operator>>,
    operands_scopes: Vec<Vec<usize>>,
    ast: Ast,
}

impl<'a> Parser<'a> {
    fn new(input: &'a TokenList) -> Self {
        Self {
            source: input,
            input: input.as_slice(),
            operators: Vec::new(),
            operands: Vec::new(),
            operators_scopes: Vec::new(),
            operands_scopes: Vec::new(),
            ast: Ast::new(),
        }
    }

    fn push_scope(&mut self) {
        let operators = std::mem::take(&mut self.operators);
        let operands = std::mem::take(&mut self.operands);

        self.operators_scopes.push(operators);
        self.operands_scopes.push(operands);
    }

    fn pop_scope(&mut self) {
        if let Some(ops) = self.operators_scopes.pop() {
            self.operators = ops;
        }

        if let Some(ops) = self.operands_scopes.pop() {
            self.operands = ops;
        }
    }

    fn parse(mut self) -> Res<Ast> {
        self.parse_first()?;
        if self.input.is_empty() {
            Ok(self.ast)
        } else {
            let token = self.input.first().unwrap();
            self.token_err(token, "Input not consumed.")
        }
    }

    fn parse_first(&mut self) -> Res<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        self.operators.push(Operator::Sentinel);
        self.expr()?;
        Ok(())
    }

    fn expr(&mut self) -> Res<usize> {
        let mut id = self.term()?;

        while let Some(token) = self.peek()
            && let Tok::Operator(op) = token.inner()
            && op.is_binary()
        {
            let op = *op;
            self.push_operator(op);
            self.next()?; // throw away token
            self.term()?;
        }

        while !matches!(self.operators.last(), Some(Operator::Sentinel))
            && !self.operators.is_empty()
        {
            id = self.pop_operator()?;
        }

        Ok(id)
    }

    fn term(&mut self) -> Res<usize> {
        let token = self.next()?.clone();
        let id = match token.inner() {
            Tok::Identifier(name) => match name.as_str() {
                "if" => self.conditional(),
                "then" | "else" => {
                    self.token_err(&token, format!("{name} must follow an opening if."))
                }
                "true" => Ok(self.push_operand(Node::Value(Value::Bool(true)))),
                "false" => Ok(self.push_operand(Node::Value(Value::Bool(false)))),
                _ => {
                    if self.next_is(Tok::ParenOpen) {
                        self.call(name.clone())
                    } else {
                        Ok(self.push_operand(Node::Identifier(name.clone())))
                    }
                }
            },
            Tok::Natural(n) => Ok(self.push_operand(Node::Value(Value::Natural(*n as i64)))),
            Tok::Decimal(v) => Ok(self.push_operand(Node::Value(Value::Decimal(*v)))),
            Tok::Roll(q, d) => Ok(self.push_operand(Node::Value(Value::Roll(Roll::new(*q, *d))))),
            Tok::String(val) => Ok(self.push_operand(Node::Value(Value::String(val.clone())))),
            Tok::ParenOpen => {
                self.operators.push(Operator::Sentinel);
                let id = self.expr()?;
                self.expect(Tok::ParenClose)?;
                self.operators.pop();
                Ok(id)
            }
            Tok::ParenClose => self.token_err(&token, ") unexpected."),
            Tok::BracketOpen => self.list(),
            Tok::BracketClose => self.token_err(&token, "] unexpected."),
            Tok::Comma => self.token_err(&token, ", unexpected."),
            Tok::Operator(op) if op.is_unary_prefix() => {
                self.push_operator(*op);
                self.term()
            }
            Tok::Operator(Operator::Sub) => {
                // N.B. sub / neg can be ambiguous, so allow sub in place of
                // neg as a unary prefix.
                self.push_operator(Operator::Neg);
                self.term()
            }
            Tok::Operator(op) => self.token_err(&token, format!("{} unexpected.", op.str())),
        }?;

        while let Some(token) = self.peek()
            && let Tok::Operator(op) = token.inner()
            && op.is_unary_postfix()
        {
            let op = *op;
            self.push_operator(op);
            self.next()?; // throw away token
        }

        Ok(id)
    }

    fn in_scope<T, F: FnOnce(&mut Self) -> Res<T>>(&mut self, func: F) -> Res<T> {
        self.push_scope();
        let ret = func(self);
        self.pop_scope();
        ret
    }

    fn conditional(&mut self) -> Res<usize> {
        let cond = self.in_scope(Self::expr)?;
        self.expect(Tok::identifier("then"))?;
        let then = self.in_scope(Self::expr)?;
        let fail = if self.next_is(Tok::identifier("else")) {
            self.next()?; // Toss else
            Some(self.in_scope(Self::expr)?)
        } else {
            None
        };
        Ok(self.push_operand(Node::If(cond, then, fail)))
    }

    fn _list(&mut self) -> Res<Node> {
        let mut values = Vec::new();
        if !self.next_is(Tok::BracketClose) {
            values.push(self.expr()?);
            while self.next_is(Tok::Comma) {
                self.expect(Tok::Comma)?;
                values.push(self.expr()?);
            }
        }
        self.expect(Tok::BracketClose)?;
        Ok(Node::List(values))
    }

    fn list(&mut self) -> Res<usize> {
        let node = self.in_scope(Self::_list)?;
        Ok(self.push_operand(node))
    }

    fn _call(&mut self, name: String) -> Res<usize> {
        self.expect(Tok::ParenOpen)?;
        let mut args = Vec::new();
        if !self.next_is(Tok::ParenClose) {
            args.push(self.expr()?);
            while self.next_is(Tok::Comma) {
                self.expect(Tok::Comma)?;
                args.push(self.expr()?);
            }
        }
        self.expect(Tok::ParenClose)?;
        Ok(self.ast.add(Node::Call(name, args)))
    }

    fn call(&mut self, name: String) -> Res<usize> {
        self.push_scope();
        let ret = self._call(name);
        self.pop_scope();
        if let Ok(id) = ret {
            self.operands.push(id);
        }
        ret
    }

    fn next(&mut self) -> Res<&Token> {
        if let Some(tok) = self.input.first() {
            self.input = &self.input[1..];
            Ok(tok)
        } else {
            err("Input ended unexpectedly.")
        }
    }

    fn expect(&mut self, tok: Tok) -> Res<()> {
        let actual = self.next()?;
        if *actual.inner() == tok {
            Ok(())
        } else {
            let token = actual.clone();
            self.token_err(
                &token,
                format!("Expected {tok:?} but found {:?}.", token.inner()),
            )
        }
    }

    fn next_is(&self, tok: Tok) -> bool {
        if let Some(token) = self.peek() {
            *token.inner() == tok
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.input.first()
    }

    fn pop_operand(&mut self) -> Res<usize> {
        if let Some(operand) = self.operands.pop() {
            Ok(operand)
        } else {
            err("Attempted to pop empty operand stack.")
        }
    }

    fn pop_operator(&mut self) -> Res<usize> {
        if let Some(op) = self.operators.pop() {
            if op.is_binary() {
                let rhs = self.pop_operand()?;
                let lhs = self.pop_operand()?;
                Ok(self.push_operand(Node::Binary(lhs, op, rhs)))
            } else if op.is_unary() {
                let arg = self.pop_operand()?;
                Ok(self.push_operand(Node::Unary(arg, op)))
            } else {
                err("Attempted to pop Sentinel operator.")
            }
        } else {
            err("Attempted to pop empty operator stack.")
        }
    }

    fn push_operand(&mut self, operand: Node) -> usize {
        let id = self.ast.add(operand);
        self.operands.push(id);
        id
    }

    fn push_operator(&mut self, op: Operator) {
        while let Some(top) = self.operators.last() {
            if Operator::greater(top, &op) {
                self.pop_operator().ok();
            } else {
                break;
            }
        }
        self.operators.push(op);
    }

    fn token_err<T, S: std::fmt::Display>(&self, token: &Token, message: S) -> Res<T> {
        Err(format!("{}\n{}", self.source.context(token), message))
    }
}

pub fn parse(input: &TokenList) -> Res<Ast> {
    Parser::new(input).parse()
}

pub fn parse_first(input: &TokenList) -> Res<(Ast, &[Token])> {
    let mut parser = Parser::new(input);
    parser.parse_first()?;
    Ok((parser.ast, parser.input))
}

pub fn parse_tome(mut input: TokenList) -> Res<Vec<Ast>> {
    let mut statements = Vec::new();
    while !input.is_empty() {
        let (ast, rest) = parse_first(&input)?;
        statements.push(ast);
        input.truncate(input.len().saturating_sub(rest.len()));
    }
    Ok(statements)
}

#[cfg(test)]
mod test {
    use crate::token::{tokenise, toks_to_list};

    use super::*;

    fn ast_of(input: &str) -> Ast {
        match parse(&tokenise(input).unwrap()) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parsing failed:\n{e}");
                panic!();
            }
        }
    }

    fn check_exprs(input: &str, expected: Vec<Node>) {
        assert_eq!(ast_of(input).exprs(), expected)
    }

    fn root(ast: &Ast) -> Option<&Node> {
        ast.get(ast.start())
    }

    fn parse_toks(toks: &[Tok]) -> Res<Ast> {
        parse(&toks_to_list(toks.into()))
    }

    #[test]
    fn test_parse_addition() {
        let ast = parse_toks(&[
            Tok::Natural(2),
            Tok::Operator(Operator::Add),
            Tok::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Add, 1)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Binary(0, Operator::Add, 1)));
    }

    #[test]
    fn test_negation() {
        let ast = parse_toks(&[Tok::Operator(Operator::Sub), Tok::Natural(3)]).unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(3)),
                Node::Unary(0, Operator::Neg)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Unary(0, Operator::Neg)));

        let ast = parse_toks(&[
            Tok::Natural(2),
            Tok::Operator(Operator::Add),
            Tok::Operator(Operator::Sub),
            Tok::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Unary(1, Operator::Neg),
                Node::Binary(0, Operator::Add, 2)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Binary(0, Operator::Add, 2)));
    }

    #[test]
    fn test_precedence() {
        let ast = parse_toks(&[
            Tok::Operator(Operator::Sub),
            Tok::Natural(2),
            Tok::Operator(Operator::Add),
            Tok::Natural(3),
            Tok::Operator(Operator::Exp),
            Tok::Natural(4),
            Tok::Operator(Operator::Mul),
            Tok::Natural(5),
            Tok::Operator(Operator::Sub),
            Tok::Natural(6),
        ])
        .unwrap();

        // -2 + 3 ^ 4 * 5 - 6 = ((-2) + ((3 ^ 4) * 5)) - 6
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(2)),
                Node::Unary(0, Operator::Neg),
                Node::Value(Value::Natural(3)),
                Node::Value(Value::Natural(4)),
                Node::Binary(2, Operator::Exp, 3),
                Node::Value(Value::Natural(5)),
                Node::Binary(4, Operator::Mul, 5),
                Node::Binary(1, Operator::Add, 6),
                Node::Value(Value::Natural(6)),
                Node::Binary(7, Operator::Sub, 8)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Binary(7, Operator::Sub, 8)));
    }

    #[test]
    fn test_neg_precedence() {
        let ast = parse_toks(&[
            Tok::Operator(Operator::Sub),
            Tok::Natural(2),
            Tok::Operator(Operator::Exp),
            Tok::Natural(3),
        ])
        .unwrap();

        assert_eq!(root(&ast), Some(&Node::Unary(2, Operator::Neg)));
    }

    #[test]
    fn test_parse_repeated_addition() {
        let ast = parse_toks(&[
            Tok::Natural(2),
            Tok::Operator(Operator::Add),
            Tok::Natural(3),
            Tok::Operator(Operator::Add),
            Tok::Natural(4),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Add, 1),
                Node::Value(Value::Natural(4)),
                Node::Binary(2, Operator::Add, 3)
            ]
        );
    }

    #[test]
    fn test_addition_subtraction() {
        let ast = parse_toks(&[
            Tok::Natural(3),
            Tok::Operator(Operator::Sub),
            Tok::Natural(4),
            Tok::Operator(Operator::Add),
            Tok::Natural(5),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Natural(3)),
                Node::Value(Value::Natural(4)),
                Node::Binary(0, Operator::Sub, 1),
                Node::Value(Value::Natural(5)),
                Node::Binary(2, Operator::Add, 3)
            ]
        );
    }

    #[test]
    fn test_keep() {
        let ast = parse_toks(&[
            Tok::Roll(10, 8),
            Tok::Operator(Operator::Keep),
            Tok::Natural(8),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Roll(Roll::new(10, 8))),
                Node::Value(Value::Natural(8)),
                Node::Binary(0, Operator::Keep, 1)
            ]
        );
    }

    #[test]
    fn test_roll_operators() {
        let ast = parse_toks(&[
            Tok::Roll(1, 20),
            Tok::Operator(Operator::Adv),
            Tok::Operator(Operator::Add),
            Tok::Roll(1, 4),
            Tok::Operator(Operator::DisAdv),
            Tok::Operator(Operator::Add),
            Tok::Roll(10, 8),
            Tok::Operator(Operator::Keep),
            Tok::Natural(8),
        ])
        .unwrap();

        // d20a + d4d + 10d8k8s = ((d20a) + (d4d)) + ((10d8k8)s)
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Value(Value::Roll(Roll::new(1, 20))),
                Node::Unary(0, Operator::Adv),
                Node::Value(Value::Roll(Roll::new(1, 4))),
                Node::Unary(2, Operator::DisAdv),
                Node::Binary(1, Operator::Add, 3),
                Node::Value(Value::Roll(Roll::new(10, 8))),
                Node::Value(Value::Natural(8)),
                Node::Binary(5, Operator::Keep, 6),
                Node::Binary(4, Operator::Add, 7)
            ]
        );
    }

    #[test]
    fn test_arithmetic() {
        check_exprs(
            "4 + 3 - 2 * 5",
            vec![
                Node::Value(Value::Natural(4)),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Add, 1),
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(5)),
                Node::Binary(3, Operator::Mul, 4),
                Node::Binary(2, Operator::Sub, 5),
            ],
        );
    }

    #[test]
    fn test_parse_exponent() {
        check_exprs(
            "-5^3*3",
            vec![
                Node::Value(Value::Natural(5)),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Exp, 1),
                Node::Unary(2, Operator::Neg),
                Node::Value(Value::Natural(3)),
                Node::Binary(3, Operator::Mul, 4),
            ],
        );
    }

    #[test]
    fn test_variables() {
        check_exprs(
            "var + 3",
            vec![
                Node::Identifier("var".into()),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Add, 1),
            ],
        )
    }

    #[test]
    fn test_variables_2() {
        check_exprs(
            "var1 * var2 ^ var3",
            vec![
                Node::Identifier("var1".into()),
                Node::Identifier("var2".into()),
                Node::Identifier("var3".into()),
                Node::Binary(1, Operator::Exp, 2),
                Node::Binary(0, Operator::Mul, 3),
            ],
        )
    }

    #[test]
    fn test_assignment() {
        check_exprs(
            "var = 2 + 3",
            vec![
                Node::Identifier("var".into()),
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Binary(1, Operator::Add, 2),
                Node::Binary(0, Operator::Assign, 3),
            ],
        )
    }

    #[test]
    fn test_simple_assignment() {
        check_exprs(
            "var = 0",
            vec![
                Node::Identifier("var".into()),
                Node::Value(Value::Natural(0)),
                Node::Binary(0, Operator::Assign, 1),
            ],
        )
    }

    #[test]
    fn test_define_assign() {
        check_exprs(
            "fn() = var = 1 + 2",
            vec![
                Node::Call("fn".into(), Vec::new()),
                Node::Identifier("var".into()),
                Node::Value(Value::Natural(1)),
                Node::Value(Value::Natural(2)),
                Node::Binary(2, Operator::Add, 3),
                Node::Binary(1, Operator::Assign, 4),
                Node::Binary(0, Operator::Assign, 5),
            ],
        )
    }

    #[test]
    fn test_call() {
        check_exprs(
            "fn(1, 2, 3)",
            vec![
                Node::Value(Value::Natural(1)),
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Call("fn".into(), vec![0, 1, 2]),
            ],
        )
    }

    #[test]
    fn test_call_empty() {
        check_exprs("fn()", vec![Node::Call("fn".into(), Vec::new())])
    }

    #[test]
    fn test_call_nested() {
        check_exprs(
            "outer(inner1(), inner2(3))",
            vec![
                Node::Call("inner1".into(), Vec::new()),
                Node::Value(Value::Natural(3)),
                Node::Call("inner2".into(), vec![1]),
                Node::Call("outer".into(), vec![0, 2]),
            ],
        )
    }

    #[test]
    fn test_call_arithmetic() {
        check_exprs(
            "5 + fn() * 2",
            vec![
                Node::Value(Value::Natural(5)),
                Node::Call("fn".into(), Vec::new()),
                Node::Value(Value::Natural(2)),
                Node::Binary(1, Operator::Mul, 2),
                Node::Binary(0, Operator::Add, 3),
            ],
        )
    }

    #[test]
    fn test_parse_definition() {
        check_exprs(
            "avg(roll) = (dice(roll) + 1) / 2",
            vec![
                Node::Identifier("roll".into()),
                Node::Call("avg".into(), vec![0]),
                Node::Identifier("roll".into()),
                Node::Call("dice".into(), vec![2]),
                Node::Value(Value::Natural(1)),
                Node::Binary(3, Operator::Add, 4),
                Node::Value(Value::Natural(2)),
                Node::Binary(5, Operator::Div, 6),
                Node::Binary(1, Operator::Assign, 7),
            ],
        )
    }

    #[test]
    fn test_parse_definition_multiple_calls() {
        check_exprs(
            "func(ina, inb) = (f1(ina, 1) + f2(inb, 2)) * 2",
            vec![
                Node::name("ina"),
                Node::name("inb"),
                Node::Call("func".into(), vec![0, 1]),
                Node::name("ina"),
                Node::Value(Value::Natural(1)),
                Node::Call("f1".into(), vec![3, 4]),
                Node::name("inb"),
                Node::Value(Value::Natural(2)),
                Node::Call("f2".into(), vec![6, 7]),
                Node::Binary(5, Operator::Add, 8),
                Node::Value(Value::Natural(2)),
                Node::Binary(9, Operator::Mul, 10),
                Node::Binary(2, Operator::Assign, 11),
            ],
        )
    }

    #[test]
    fn test_parse_list() {
        check_exprs(
            "[1, 2, 3]",
            vec![
                Node::Value(Value::Natural(1)),
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::List(vec![0, 1, 2]),
            ],
        )
    }

    #[test]
    fn test_parse_list_of_exprs() {
        check_exprs(
            "[[\"a\", 1], 8d8]",
            vec![
                Node::Value(Value::String("a".into())),
                Node::Value(Value::Natural(1)),
                Node::List(vec![0, 1]),
                Node::Value(Value::Roll(Roll::new(8, 8))),
                Node::List(vec![2, 3]),
            ],
        )
    }

    #[test]
    fn test_parse_if() {
        check_exprs(
            "if true then 1",
            vec![
                Node::Value(Value::Bool(true)),
                Node::Value(Value::Natural(1)),
                Node::If(0, 1, None),
            ],
        )
    }

    #[test]
    fn test_parse_if_else() {
        check_exprs(
            "if false then 1 else 2",
            vec![
                Node::Value(Value::Bool(false)),
                Node::Value(Value::Natural(1)),
                Node::Value(Value::Natural(2)),
                Node::If(0, 1, Some(2)),
            ],
        )
    }

    #[test]
    fn test_parse_if_in_func() {
        check_exprs(
            "fn(x, y, z) = if x then y else z",
            vec![
                Node::name("x"),
                Node::name("y"),
                Node::name("z"),
                Node::Call("fn".into(), vec![0, 1, 2]),
                Node::name("x"),
                Node::name("y"),
                Node::name("z"),
                Node::If(4, 5, Some(6)),
                Node::Binary(3, Operator::Assign, 7),
            ],
        )
    }

    #[test]
    fn test_parse_complex_if_condition() {
        check_exprs(
            "if 2 + 3 > 4 then \"yes\" else \"no\"",
            vec![
                Node::Value(Value::Natural(2)),
                Node::Value(Value::Natural(3)),
                Node::Binary(0, Operator::Add, 1),
                Node::Value(Value::Natural(4)),
                Node::Binary(2, Operator::GreaterThan, 3),
                Node::Value(Value::String("yes".into())),
                Node::Value(Value::String("no".into())),
                Node::If(4, 5, Some(6)),
            ],
        )
    }

    #[test]
    fn test_function_returning_list() {
        check_exprs(
            r#"p() = [print("a")]"#,
            vec![
                Node::Call("p".into(), Vec::new()),
                Node::Value(Value::String("a".into())),
                Node::Call("print".into(), vec![1]),
                Node::List(vec![2]),
                Node::Binary(0, Operator::Assign, 3),
            ],
        )
    }

    #[test]
    fn test_not_operator() {
        check_exprs(
            "if !a then b = b + 1",
            vec![
                Node::Identifier("a".into()),
                Node::Unary(0, Operator::Not),
                Node::Identifier("b".into()),
                Node::Identifier("b".into()),
                Node::Value(Value::Natural(1)),
                Node::Binary(3, Operator::Add, 4),
                Node::Binary(2, Operator::Assign, 5),
                Node::If(1, 6, None),
            ],
        )
    }

    #[test]
    fn test_complicated_if() {
        check_exprs(
            "if (_sp > 0 | _ep > 0 | _gp > 0 | _pp > 0) & spend_sp(1) then true",
            vec![
                Node::name("_sp"),
                Node::Value(Value::Natural(0)),
                Node::Binary(0, Operator::GreaterThan, 1),
                Node::name("_ep"),
                Node::Value(Value::Natural(0)),
                Node::Binary(3, Operator::GreaterThan, 4),
                Node::Binary(2, Operator::Or, 5),
                Node::name("_gp"),
                Node::Value(Value::Natural(0)),
                Node::Binary(7, Operator::GreaterThan, 8),
                Node::Binary(6, Operator::Or, 9),
                Node::name("_pp"),
                Node::Value(Value::Natural(0)),
                Node::Binary(11, Operator::GreaterThan, 12),
                Node::Binary(10, Operator::Or, 13),
                Node::Value(Value::Natural(1)),
                Node::Call("spend_sp".into(), vec![15]),
                Node::Binary(14, Operator::And, 16),
                Node::Value(Value::Bool(true)),
                Node::If(17, 18, None),
            ],
        )
    }
}
