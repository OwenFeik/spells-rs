use super::token::Token;

/// Grammar
/// expr := term { binary term }
/// term := factor | ( expr ) | unary-prefix term | term unary-suffix
/// binary := + | - | * | / | ^ | k
/// unary := -
/// factor := Ra | Rd | Rs | R | N
/// R := NdN | dN
/// N := NN | [0-9]
///
/// TODO unary-suffix

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Adv(Box<Expr>),
    DisAdv(Box<Expr>),
    Sort(Box<Expr>),
    Keep(Box<Expr>, Box<Expr>),
    Roll(u32, u32),
    Natural(u32),
}

impl Expr {
    fn add(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Add(Box::new(lhs), Box::new(rhs))
    }

    fn sub(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Sub(Box::new(lhs), Box::new(rhs))
    }

    fn mul(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Mul(Box::new(lhs), Box::new(rhs))
    }

    fn div(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Div(Box::new(lhs), Box::new(rhs))
    }

    fn exp(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Exp(Box::new(lhs), Box::new(rhs))
    }

    fn keep(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Keep(Box::new(lhs), Box::new(rhs))
    }

    fn neg(operand: Expr) -> Expr {
        Expr::Neg(Box::new(operand))
    }

    fn adv(operand: Expr) -> Expr {
        Expr::Adv(Box::new(operand))
    }

    fn disadv(operand: Expr) -> Expr {
        Expr::DisAdv(Box::new(operand))
    }

    fn sort(operand: Expr) -> Expr {
        Expr::Sort(Box::new(operand))
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
            Operator::Exp => 3,
            Operator::Neg => 5,
            Operator::Keep => 4,
            Operator::Adv => 5,
            Operator::DisAdv => 5,
            Operator::Sort => 5,
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

    fn binary(self, lhs: Expr, rhs: Expr) -> ParseResult<Expr> {
        match self {
            Operator::Add => Ok(Expr::add(lhs, rhs)),
            Operator::Sub => Ok(Expr::sub(lhs, rhs)),
            Operator::Mul => Ok(Expr::mul(lhs, rhs)),
            Operator::Div => Ok(Expr::div(lhs, rhs)),
            Operator::Exp => Ok(Expr::exp(lhs, rhs)),
            Operator::Keep => Ok(Expr::keep(lhs, rhs)),
            _ => err(format!("{self:?} is not a binary operator.")),
        }
    }

    fn is_unary(&self) -> bool {
        matches!(
            self,
            Operator::Neg | Operator::Keep | Operator::Adv | Operator::DisAdv | Operator::Sort
        )
    }

    fn unary(self, operand: Expr) -> ParseResult<Expr> {
        match self {
            Operator::Neg => Ok(Expr::neg(operand)),
            Operator::Adv => Ok(Expr::adv(operand)),
            Operator::DisAdv => Ok(Expr::disadv(operand)),
            Operator::Sort => Ok(Expr::sort(operand)),
            _ => err(format!("{self:?} is not a unary operator.")),
        }
    }

    fn greater(left: &Self, right: &Self) -> bool {
        if left.is_binary() && right.is_binary() {
            (left.precedence() > right.precedence())
                || (left.left_associative() && left.precedence() == right.precedence())
        } else if left.is_unary() && right.is_binary() {
            left.precedence() >= right.precedence()
        } else {
            false
        }
    }

    fn from(token: &Token) -> ParseResult<Self> {
        match token {
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
    operands: Vec<Expr>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [Token]) -> Self {
        Self {
            input,
            operators: Vec::new(),
            operands: Vec::new(),
        }
    }

    fn parse(mut self) -> ParseResult<Expr> {
        self.operators.push(Operator::Sentinel);
        self.expr()?;
        if self.input.is_empty() {
            if let Some(expr) = self.operands.pop() {
                Ok(expr)
            } else {
                err("Operand stack empty at parse conclusion.")
            }
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
        match *self.next()? {
            Token::Natural(n) => {
                self.operands.push(Expr::Natural(n));
                Ok(())
            }
            Token::Roll(q, d) => {
                self.operands.push(Expr::Roll(q, d));
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
        }
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

    fn pop_operand(&mut self) -> ParseResult<Expr> {
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
                self.operands.push(op.binary(lhs, rhs)?);
                Ok(())
            } else if op.is_unary() {
                let operand = self.pop_operand()?;
                self.operands.push(op.unary(operand)?);
                Ok(())
            } else {
                err("Attempted to pop Sentinel operator.")
            }
        } else {
            err("Attempted to pop empty operator stack.")
        }
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

pub fn parse(input: &[Token]) -> Result<Expr, String> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_addition() {
        let expr = parse(&[Token::Natural(2), Token::Plus, Token::Natural(3)]).unwrap();
        assert_eq!(expr, Expr::add(Expr::Natural(2), Expr::Natural(3)));
    }

    #[test]
    fn test_negation() {
        let expr = parse(&[Token::Minus, Token::Natural(3)]).unwrap();
        assert_eq!(expr, Expr::neg(Expr::Natural(3)));

        let expr = parse(&[
            Token::Natural(2),
            Token::Plus,
            Token::Minus,
            Token::Natural(3),
        ])
        .unwrap();
        assert_eq!(
            expr,
            Expr::add(Expr::Natural(2), Expr::neg(Expr::Natural(3)))
        );
    }

    #[test]
    fn test_precedence() {
        let expr = parse(&[
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
            expr,
            Expr::sub(
                Expr::add(
                    Expr::neg(Expr::Natural(2)),
                    Expr::mul(
                        Expr::exp(Expr::Natural(3), Expr::Natural(4)),
                        Expr::Natural(5)
                    )
                ),
                Expr::Natural(6)
            )
        );
    }

    #[test]
    fn test_parse_repexpected_addition() {
        let expr = parse(&[
            Token::Natural(2),
            Token::Plus,
            Token::Natural(3),
            Token::Plus,
            Token::Natural(4),
        ])
        .unwrap();
        let lhs = Expr::add(Expr::Natural(2), Expr::Natural(3));
        assert_eq!(expr, Expr::add(lhs, Expr::Natural(4)));
    }

    #[test]
    fn test_addition_subtraction() {
        let expr = parse(&[
            Token::Natural(3),
            Token::Minus,
            Token::Natural(4),
            Token::Plus,
            Token::Natural(5),
        ])
        .unwrap();
        assert_eq!(
            expr,
            Expr::add(
                Expr::sub(Expr::Natural(3), Expr::Natural(4)),
                Expr::Natural(5),
            )
        )
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
    fn test_roll_operators() {
        let expr = parse(&[
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
            expr,
            Expr::add(
                Expr::add(Expr::adv(Expr::Roll(1, 20)), Expr::disadv(Expr::Roll(1, 4))),
                Expr::sort(Expr::keep(Expr::Roll(10, 8), Expr::Natural(8)))
            )
        );
    }
}
