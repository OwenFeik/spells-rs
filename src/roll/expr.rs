use super::token::Token;

/// Grammar
/// expr := term { binary term }
/// term := factor | ( expr ) | unary-prefix term | term unary-postfix
/// binary := + | - | * | / | ^ | k
/// unary-prefix := -
/// unary-postfix := a | d | s | k
/// factor := R | N
/// R := NdN | dN
/// N := NN | [0-9]

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Exp(usize, usize),
    Neg(usize),
    Adv(usize),
    DisAdv(usize),
    Sort(usize),
    Keep(usize, usize),
    Roll(u32, u32),
    Natural(u32),
}

#[derive(Debug)]
pub struct Ast(Vec<Expr>);

impl Ast {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, expr: Expr) -> usize {
        self.0.push(expr);
        self.0.len() - 1
    }

    pub fn get(&self, expr: usize) -> Option<&Expr> {
        self.0.get(expr)
    }

    pub fn start(&self) -> usize {
        if self.0.is_empty() {
            0
        } else {
            self.0.len() - 1
        }
    }
}

type ParseErr = String;
type ParseResult<T> = Result<T, ParseErr>;

fn err<T, S: ToString>(msg: S) -> ParseResult<T> {
    Err(msg.to_string())
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    Sentinel,
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Neg,
    Keep,
    Adv,
    DisAdv,
    Sort,
}

impl Operator {
    fn precedence(&self) -> u8 {
        match self {
            Operator::Sentinel => 0,
            Operator::Add => 1,
            Operator::Sub => 1,
            Operator::Mul => 2,
            Operator::Div => 2,
            Operator::Exp => 4,
            Operator::Neg => 3,
            Operator::Keep => 5,
            Operator::Adv => 3,
            Operator::DisAdv => 3,
            Operator::Sort => 3,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => false,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
            Operator::Sort => false,
        }
    }

    fn is_binary(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => true,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
            Operator::Sort => false,
        }
    }

    fn binary(self, lhs: usize, rhs: usize) -> ParseResult<Expr> {
        match self {
            Operator::Add => Ok(Expr::Add(lhs, rhs)),
            Operator::Sub => Ok(Expr::Sub(lhs, rhs)),
            Operator::Mul => Ok(Expr::Mul(lhs, rhs)),
            Operator::Div => Ok(Expr::Div(lhs, rhs)),
            Operator::Exp => Ok(Expr::Exp(lhs, rhs)),
            Operator::Keep => Ok(Expr::Keep(lhs, rhs)),
            _ => err(format!("{self:?} is not a binary operator.")),
        }
    }

    fn is_unary(&self) -> bool {
        matches!(
            self,
            Operator::Neg | Operator::Adv | Operator::DisAdv | Operator::Sort
        )
    }

    fn is_unary_postfix(&self) -> bool {
        matches!(self, Operator::Adv | Operator::DisAdv | Operator::Sort)
    }

    fn unary(self, operand: usize) -> ParseResult<Expr> {
        match self {
            Operator::Neg => Ok(Expr::Neg(operand)),
            Operator::Adv => Ok(Expr::Adv(operand)),
            Operator::DisAdv => Ok(Expr::DisAdv(operand)),
            Operator::Sort => Ok(Expr::Sort(operand)),
            _ => err(format!("{self:?} is not a unary operator.")),
        }
    }

    fn greater(left: &Self, right: &Self) -> bool {
        if left.is_binary() && right.is_binary() {
            (left.precedence() > right.precedence())
                || (left.left_associative() && left.precedence() == right.precedence())
        } else if left.is_unary() && right.is_binary() || right.is_unary_postfix() {
            left.precedence() >= right.precedence()
        } else {
            false
        }
    }

    fn from(token: &Token) -> ParseResult<Self> {
        match token {
            Token::Identifier(name) => err(format!("{name} is not an operator.")),
            Token::Natural(_) => err("Natural is not an operator."),
            Token::Roll(_, _) => err("Roll is not an operator."),
            Token::ParenOpen => err("( is not an operator."),
            Token::ParenClose => err(") is not an operator."),
            Token::Plus => Ok(Self::Add),
            Token::Minus => Ok(Self::Sub),
            Token::Times => Ok(Self::Mul),
            Token::Divide => Ok(Self::Div),
            Token::Exp => Ok(Self::Exp),
            Token::Keep => Ok(Self::Keep),
            Token::Advantage => Ok(Self::Adv),
            Token::Disadvantage => Ok(Self::DisAdv),
            Token::Sort => Ok(Self::Sort),
        }
    }
}

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

    fn parse(mut self) -> ParseResult<Ast> {
        self.operators.push(Operator::Sentinel);
        self.expr()?;
        if self.input.is_empty() {
            Ok(self.ast)
        } else {
            err("Input not consumed.")
        }
    }

    fn expr(&mut self) -> ParseResult<()> {
        self.term()?;

        while let Some(Ok(op)) = self.peek().map(Operator::from) && op.is_binary() {
            self.push_operator(op);
            self.next()?; // throw away token
            self.term()?;
        }

        while !matches!(self.operators.last(), Some(Operator::Sentinel))
            && !self.operators.is_empty()
        {
            self.pop_operator()?;
        }

        Ok(())
    }

    fn term(&mut self) -> ParseResult<()> {
        let res = match *self.next()? {
            Token::Identifier(_) => { Ok(()) }
            Token::Natural(n) => {
                self.operands.push(self.ast.add(Expr::Natural(n)));
                Ok(())
            }
            Token::Roll(q, d) => {
                self.operands.push(self.ast.add(Expr::Roll(q, d)));
                Ok(())
            }
            Token::ParenOpen => {
                self.operators.push(Operator::Sentinel);
                self.expr()?;
                self.expect(Token::ParenClose)?;
                self.operators.pop();
                Ok(())
            }
            Token::ParenClose => err(") unexpected."),
            Token::Plus => err("+ unexpected."),
            Token::Minus => {
                self.push_operator(Operator::Neg);
                self.term()
            }
            Token::Times => err("* unexpected."),
            Token::Divide => err("/ unexpected."),
            Token::Exp => err("^ unexpected."),
            Token::Keep => err("k unexpected."),
            Token::Advantage => err("a unexpected."),
            Token::Disadvantage => err("d unexpected."),
            Token::Sort => err("s unexpected."),
        };
        res?;

        while let Some(Ok(op)) = self.peek().map(Operator::from) && op.is_unary_postfix() {
            self.push_operator(op);
            self.next()?; // throw away token
        }

        Ok(())
    }

    fn next(&mut self) -> ParseResult<&Token> {
        if let Some(tok) = self.input.first() {
            self.input = &self.input[1..];
            Ok(tok)
        } else {
            err("Input ended unexpectedly.")
        }
    }

    fn expect(&mut self, token: Token) -> ParseResult<()> {
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

    fn pop_operand(&mut self) -> ParseResult<usize> {
        if let Some(operand) = self.operands.pop() {
            Ok(operand)
        } else {
            err("Attempted to pop empty operand stack.")
        }
    }

    fn pop_operator(&mut self) -> ParseResult<()> {
        if let Some(op) = self.operators.pop() {
            if op.is_binary() {
                let rhs = self.pop_operand()?;
                let lhs = self.pop_operand()?;
                self.push_operand(op.binary(lhs, rhs)?);
                Ok(())
            } else if op.is_unary() {
                let operand = self.pop_operand()?;
                self.push_operand(op.unary(operand)?);
                Ok(())
            } else {
                err("Attempted to pop Sentinel operator.")
            }
        } else {
            err("Attempted to pop empty operator stack.")
        }
    }

    fn push_operand(&mut self, operand: Expr) {
        self.operands.push(self.ast.add(operand));
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

pub fn lex(input: &[Token]) -> Result<Ast, String> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use crate::roll::token::tokenise;

    use super::*;

    fn exprs(ast: &Ast) -> &[Expr] {
        &ast.0
    }

    fn root(ast: &Ast) -> Option<&Expr> {
        ast.get(ast.start())
    }

    #[test]
    fn test_parse_addition() {
        let ast = lex(&[Token::Natural(2), Token::Plus, Token::Natural(3)]).unwrap();
        assert_eq!(
            exprs(&ast),
            vec![Expr::Natural(2), Expr::Natural(3), Expr::Add(0, 1)]
        );
        assert_eq!(root(&ast), Some(&Expr::Add(0, 1)));
    }

    #[test]
    fn test_negation() {
        let ast = lex(&[Token::Minus, Token::Natural(3)]).unwrap();
        assert_eq!(exprs(&ast), vec![Expr::Natural(3), Expr::Neg(0)]);
        assert_eq!(root(&ast), Some(&Expr::Neg(0)));

        let ast = lex(&[
            Token::Natural(2),
            Token::Plus,
            Token::Minus,
            Token::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            exprs(&ast),
            vec![
                Expr::Natural(2),
                Expr::Natural(3),
                Expr::Neg(1),
                Expr::Add(0, 2)
            ]
        );
        assert_eq!(root(&ast), Some(&Expr::Add(0, 2)));
    }

    #[test]
    fn test_precedence() {
        let ast = lex(&[
            Token::Minus,
            Token::Natural(2),
            Token::Plus,
            Token::Natural(3),
            Token::Exp,
            Token::Natural(4),
            Token::Times,
            Token::Natural(5),
            Token::Minus,
            Token::Natural(6),
        ])
        .unwrap();

        // -2 + 3 * 4 - 5 = ((-2) + ((3 ^ 4) * 5)) - 6
        assert_eq!(
            exprs(&ast),
            vec![
                Expr::Natural(2),
                Expr::Neg(0),
                Expr::Natural(3),
                Expr::Natural(4),
                Expr::Exp(2, 3),
                Expr::Natural(5),
                Expr::Mul(4, 5),
                Expr::Add(1, 6),
                Expr::Natural(6),
                Expr::Sub(7, 8)
            ]
        );
        assert_eq!(root(&ast), Some(&Expr::Sub(7, 8)));
    }

    #[test]
    fn test_neg_precedence() {
        let ast = lex(&[
            Token::Minus,
            Token::Natural(2),
            Token::Exp,
            Token::Natural(3),
        ])
        .unwrap();

        assert_eq!(root(&ast), Some(&Expr::Neg(2)));
    }

    #[test]
    fn test_parse_repeated_addition() {
        let ast = lex(&[
            Token::Natural(2),
            Token::Plus,
            Token::Natural(3),
            Token::Plus,
            Token::Natural(4),
        ])
        .unwrap();
        assert_eq!(
            exprs(&ast),
            vec![
                Expr::Natural(2),
                Expr::Natural(3),
                Expr::Add(0, 1),
                Expr::Natural(4),
                Expr::Add(2, 3)
            ]
        );
    }

    #[test]
    fn test_addition_subtraction() {
        let ast = lex(&[
            Token::Natural(3),
            Token::Minus,
            Token::Natural(4),
            Token::Plus,
            Token::Natural(5),
        ])
        .unwrap();
        assert_eq!(
            exprs(&ast),
            vec![
                Expr::Natural(3),
                Expr::Natural(4),
                Expr::Sub(0, 1),
                Expr::Natural(5),
                Expr::Add(2, 3)
            ]
        );
    }

    #[test]
    fn test_binary_operators() {
        let token = Some(&Token::Plus);
        if let Some(Ok(op)) = token.map(Operator::from) && op.is_binary() {
            assert!(matches!(op, Operator::Add));
        } else {
            panic!();
        }
    }

    #[test]
    fn test_keep() {
        let ast = lex(&[Token::Roll(10, 8), Token::Keep, Token::Natural(8)]).unwrap();
        assert_eq!(
            exprs(&ast),
            vec![Expr::Roll(10, 8), Expr::Natural(8), Expr::Keep(0, 1)]
        );
    }

    #[test]
    fn test_roll_operators() {
        let ast = lex(&[
            Token::Roll(1, 20),
            Token::Advantage,
            Token::Plus,
            Token::Roll(1, 4),
            Token::Disadvantage,
            Token::Plus,
            Token::Roll(10, 8),
            Token::Keep,
            Token::Natural(8),
            Token::Sort,
        ])
        .unwrap();

        // d20a + d4d + 10d8k8s = ((d20a) + (d4d)) + ((10d8k8)s)
        assert_eq!(
            exprs(&ast),
            vec![
                Expr::Roll(1, 20),
                Expr::Adv(0),
                Expr::Roll(1, 4),
                Expr::DisAdv(2),
                Expr::Add(1, 3),
                Expr::Roll(10, 8),
                Expr::Natural(8),
                Expr::Keep(5, 6),
                Expr::Sort(7),
                Expr::Add(4, 8)
            ]
        );
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(
            exprs(&lex(&tokenise("4 + 3 - 2 * 5")).unwrap()),
            vec![
                Expr::Natural(4),
                Expr::Natural(3),
                Expr::Add(0, 1),
                Expr::Natural(2),
                Expr::Natural(5),
                Expr::Mul(3, 4),
                Expr::Sub(2, 5)
            ]
        );
    }

    #[test]
    fn test_parse_exponent() {
        assert_eq!(
            exprs(&lex(&tokenise("-5^3*3")).unwrap()),
            vec![
                Expr::Natural(5),
                Expr::Natural(3),
                Expr::Exp(0, 1),
                Expr::Neg(2),
                Expr::Natural(3),
                Expr::Mul(3, 4)
            ]
        );
    }
}
