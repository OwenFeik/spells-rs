use crate::{
    ast::{Ast, Node},
    err,
    operator::Operator,
    Res,
};

use super::token::Token;

struct Parser<'a> {
    input: &'a [Token],
    operators: Vec<Operator>,
    operands: Vec<usize>,
    ast: Ast,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [Token]) -> Self {
        Self {
            input,
            operators: Vec::new(),
            operands: Vec::new(),
            ast: Ast::new(),
        }
    }

    fn parse(mut self) -> Res<Ast> {
        self.operators.push(Operator::Sentinel);
        self.expr()?;
        if self.input.is_empty() {
            Ok(self.ast)
        } else {
            err("Input not consumed.")
        }
    }

    fn expr(&mut self) -> Res<usize> {
        let mut id = self.term()?;

        while let Some(Token::Operator(op)) = self.peek()
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
        let id = match self.next()?.clone() {
            Token::Identifier(name) => {
                if let Some(Token::ParenOpen) = self.peek() {
                    self.call(name)
                } else {
                    let id = self.ast.add(Node::Identifier(name));
                    self.operands.push(id);
                    Ok(id)
                }
            }
            Token::Natural(n) => Ok(self.push_operand(Node::Natural(n))),
            Token::Roll(q, d) => Ok(self.push_operand(Node::Roll(q, d))),
            Token::ParenOpen => {
                self.operators.push(Operator::Sentinel);
                let id = self.expr()?;
                self.expect(Token::ParenClose)?;
                self.operators.pop();
                Ok(id)
            }
            Token::ParenClose => err(") unexpected."),
            Token::Comma => err(", unexpected."),
            Token::Operator(Operator::Sub) | Token::Operator(Operator::Neg) => {
                self.push_operator(Operator::Neg);
                self.term()
            }
            Token::Operator(op) => err(format!("{} unexpected.", op.char())),
        }?;

        while let Some(Token::Operator(op)) = self.peek()
            && op.is_unary_postfix()
        {
            let op = *op;
            self.push_operator(op);
            self.next()?; // throw away token
        }

        Ok(id)
    }

    fn _call(&mut self, name: String) -> Res<usize> {
        self.expect(Token::ParenOpen)?;
        let mut args = Vec::new();
        if !matches!(self.peek(), Some(Token::ParenClose)) {
            args.push(self.expr()?);
            while matches!(self.peek(), Some(Token::Comma)) {
                self.expect(Token::Comma)?;
                args.push(self.expr()?);
            }
        }
        self.expect(Token::ParenClose)?;
        Ok(self.ast.add(Node::Call(name, args)))
    }

    fn call(&mut self, name: String) -> Res<usize> {
        let operators = std::mem::take(&mut self.operators);
        let operands = std::mem::take(&mut self.operands);
        let ret = self._call(name);
        self.operators = operators;
        self.operands = operands;
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

    fn expect(&mut self, token: Token) -> Res<()> {
        let actual = self.next()?;
        if actual == &token {
            Ok(())
        } else {
            err(format!("Expected {token:?} but found {actual:?}."))
        }
    }

    fn peek(&mut self) -> Option<&Token> {
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
                Ok(self.push_operand(op.binary(lhs, rhs)?))
            } else if op.is_unary() {
                let operand = self.pop_operand()?;
                Ok(self.push_operand(op.unary(operand)?))
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
}

pub fn parse(input: &[Token]) -> Res<Ast> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use crate::token::tokenise;

    use super::*;

    fn ast_of(input: &str) -> Ast {
        parse(&tokenise(input).unwrap()).unwrap()
    }

    fn check_exprs(input: &str, expected: Vec<Node>) {
        assert_eq!(ast_of(input).exprs(), expected)
    }

    fn root(ast: &Ast) -> Option<&Node> {
        ast.get(ast.start())
    }

    #[test]
    fn test_parse_addition() {
        let ast = parse(&[
            Token::Natural(2),
            Token::Operator(Operator::Add),
            Token::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![Node::Natural(2), Node::Natural(3), Node::Add(0, 1)]
        );
        assert_eq!(root(&ast), Some(&Node::Add(0, 1)));
    }

    #[test]
    fn test_negation() {
        let ast = parse(&[Token::Operator(Operator::Sub), Token::Natural(3)]).unwrap();
        assert_eq!(ast.exprs(), vec![Node::Natural(3), Node::Neg(0)]);
        assert_eq!(root(&ast), Some(&Node::Neg(0)));

        let ast = parse(&[
            Token::Natural(2),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::Sub),
            Token::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Natural(2),
                Node::Natural(3),
                Node::Neg(1),
                Node::Add(0, 2)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Add(0, 2)));
    }

    #[test]
    fn test_precedence() {
        let ast = parse(&[
            Token::Operator(Operator::Sub),
            Token::Natural(2),
            Token::Operator(Operator::Add),
            Token::Natural(3),
            Token::Operator(Operator::Exp),
            Token::Natural(4),
            Token::Operator(Operator::Mul),
            Token::Natural(5),
            Token::Operator(Operator::Sub),
            Token::Natural(6),
        ])
        .unwrap();

        // -2 + 3 * 4 - 5 = ((-2) + ((3 ^ 4) * 5)) - 6
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Natural(2),
                Node::Neg(0),
                Node::Natural(3),
                Node::Natural(4),
                Node::Exp(2, 3),
                Node::Natural(5),
                Node::Mul(4, 5),
                Node::Add(1, 6),
                Node::Natural(6),
                Node::Sub(7, 8)
            ]
        );
        assert_eq!(root(&ast), Some(&Node::Sub(7, 8)));
    }

    #[test]
    fn test_neg_precedence() {
        let ast = parse(&[
            Token::Operator(Operator::Sub),
            Token::Natural(2),
            Token::Operator(Operator::Exp),
            Token::Natural(3),
        ])
        .unwrap();

        assert_eq!(root(&ast), Some(&Node::Neg(2)));
    }

    #[test]
    fn test_parse_repeated_addition() {
        let ast = parse(&[
            Token::Natural(2),
            Token::Operator(Operator::Add),
            Token::Natural(3),
            Token::Operator(Operator::Add),
            Token::Natural(4),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Natural(2),
                Node::Natural(3),
                Node::Add(0, 1),
                Node::Natural(4),
                Node::Add(2, 3)
            ]
        );
    }

    #[test]
    fn test_addition_subtraction() {
        let ast = parse(&[
            Token::Natural(3),
            Token::Operator(Operator::Sub),
            Token::Natural(4),
            Token::Operator(Operator::Add),
            Token::Natural(5),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Natural(3),
                Node::Natural(4),
                Node::Sub(0, 1),
                Node::Natural(5),
                Node::Add(2, 3)
            ]
        );
    }

    #[test]
    fn test_keep() {
        let ast = parse(&[
            Token::Roll(10, 8),
            Token::Operator(Operator::Keep),
            Token::Natural(8),
        ])
        .unwrap();
        assert_eq!(
            ast.exprs(),
            vec![Node::Roll(10, 8), Node::Natural(8), Node::Keep(0, 1)]
        );
    }

    #[test]
    fn test_roll_operators() {
        let ast = parse(&[
            Token::Roll(1, 20),
            Token::Operator(Operator::Adv),
            Token::Operator(Operator::Add),
            Token::Roll(1, 4),
            Token::Operator(Operator::DisAdv),
            Token::Operator(Operator::Add),
            Token::Roll(10, 8),
            Token::Operator(Operator::Keep),
            Token::Natural(8),
            Token::Operator(Operator::Sort),
        ])
        .unwrap();

        // d20a + d4d + 10d8k8s = ((d20a) + (d4d)) + ((10d8k8)s)
        assert_eq!(
            ast.exprs(),
            vec![
                Node::Roll(1, 20),
                Node::Adv(0),
                Node::Roll(1, 4),
                Node::DisAdv(2),
                Node::Add(1, 3),
                Node::Roll(10, 8),
                Node::Natural(8),
                Node::Keep(5, 6),
                Node::Sort(7),
                Node::Add(4, 8)
            ]
        );
    }

    #[test]
    fn test_arithmetic() {
        check_exprs(
            "4 + 3 - 2 * 5",
            vec![
                Node::Natural(4),
                Node::Natural(3),
                Node::Add(0, 1),
                Node::Natural(2),
                Node::Natural(5),
                Node::Mul(3, 4),
                Node::Sub(2, 5),
            ],
        );
    }

    #[test]
    fn test_parse_exponent() {
        check_exprs(
            "-5^3*3",
            vec![
                Node::Natural(5),
                Node::Natural(3),
                Node::Exp(0, 1),
                Node::Neg(2),
                Node::Natural(3),
                Node::Mul(3, 4),
            ],
        );
    }

    #[test]
    fn test_variables() {
        check_exprs(
            "var + 3",
            vec![
                Node::Identifier("var".into()),
                Node::Natural(3),
                Node::Add(0, 1),
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
                Node::Exp(1, 2),
                Node::Mul(0, 3),
            ],
        )
    }

    #[test]
    fn test_assignment() {
        check_exprs(
            "var = 2 + 3",
            vec![
                Node::Identifier("var".into()),
                Node::Natural(2),
                Node::Natural(3),
                Node::Add(1, 2),
                Node::Assign(0, 3),
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
                Node::Natural(1),
                Node::Natural(2),
                Node::Add(2, 3),
                Node::Assign(1, 4),
                Node::Assign(0, 5),
            ],
        )
    }

    #[test]
    fn test_call() {
        check_exprs(
            "fn(1, 2, 3)",
            vec![
                Node::Natural(1),
                Node::Natural(2),
                Node::Natural(3),
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
                Node::Natural(3),
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
                Node::Natural(5),
                Node::Call("fn".into(), Vec::new()),
                Node::Natural(2),
                Node::Mul(1, 2),
                Node::Add(0, 3),
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
                Node::Natural(1),
                Node::Add(3, 4),
                Node::Natural(2),
                Node::Div(5, 6),
                Node::Assign(1, 7),
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
                Node::Natural(1),
                Node::Call("f1".into(), vec![3, 4]),
                Node::name("inb"),
                Node::Natural(2),
                Node::Call("f2".into(), vec![6, 7]),
                Node::Add(5, 8),
                Node::Natural(2),
                Node::Mul(9, 10),
                Node::Assign(2, 11),
            ],
        )
    }
}
