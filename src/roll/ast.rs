use super::token::Token;

// Grammar
// expr := term { binary term }
// term := factor | ( expr ) | unary-prefix term | term unary-postfix | call
// binary := + | - | * | / | ^ | k | = | :=
// call := identifier ( expr { , expr } )
// unary-postfix := a | d | s | k
// unary-prefix := -
// factor := roll | number | identfifier

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Assign(usize, usize),
    Call(String, Vec<usize>),
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
    Identifier(String),
}

impl Node {
    fn renumber_binary(&self, lhs: usize, rhs: usize) -> Self {
        match self {
            Node::Assign(_, _) => Node::Assign(lhs, rhs),
            Node::Add(_, _) => Node::Add(lhs, rhs),
            Node::Sub(_, _) => Node::Sub(lhs, rhs),
            Node::Mul(_, _) => Node::Mul(lhs, rhs),
            Node::Div(_, _) => Node::Div(lhs, rhs),
            Node::Exp(_, _) => Node::Exp(lhs, rhs),
            Node::Keep(_, _) => Node::Keep(lhs, rhs),
            _ => self.clone(),
        }
    }

    fn renumber_unary(&self, arg: usize) -> Self {
        match self {
            Node::Neg(_) => Node::Neg(arg),
            Node::Adv(_) => Node::Adv(arg),
            Node::DisAdv(_) => Node::DisAdv(arg),
            Node::Sort(_) => Node::Sort(arg),
            _ => self.clone(),
        }
    }

    fn copy(&self, from: &Ast, to: &mut Ast) -> Option<usize> {
        match self {
            &Node::Assign(lhs, rhs)
            | &Node::Add(lhs, rhs)
            | &Node::Sub(lhs, rhs)
            | &Node::Mul(lhs, rhs)
            | &Node::Div(lhs, rhs)
            | &Node::Exp(lhs, rhs)
            | &Node::Keep(lhs, rhs) => {
                let lhs = from.get(lhs)?.copy(from, to)?;
                let rhs = from.get(rhs)?.copy(from, to)?;
                Some(to.add(self.renumber_binary(lhs, rhs)))
            }
            &Node::Neg(arg) | &Node::Adv(arg) | &Node::DisAdv(arg) | &Node::Sort(arg) => {
                let arg = from.get(arg)?.copy(from, to)?;
                Some(to.add(self.renumber_unary(arg)))
            }
            Node::Call(name, args) => {
                let mut new_args = Vec::new();
                for &arg in args {
                    new_args.push(from.get(arg)?.copy(from, to)?);
                }
                Some(to.add(Node::Call(name.clone(), new_args)))
            }
            Node::Roll(_, _) | Node::Natural(_) | Node::Identifier(_) => Some(to.add(self.clone())),
        }
    }
}

#[derive(Debug)]
pub struct Ast(Vec<Node>);

impl Ast {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, expr: Node) -> usize {
        self.0.push(expr);
        self.0.len() - 1
    }

    pub fn get(&self, expr: usize) -> Option<&Node> {
        self.0.get(expr)
    }

    pub fn start(&self) -> usize {
        if self.0.is_empty() {
            0
        } else {
            self.0.len() - 1
        }
    }

    pub fn subtree(&self, root: usize) -> Option<Ast> {
        let mut subtree = Ast::new();
        self.get(root)?.copy(self, &mut subtree);
        Some(subtree)
    }

    pub fn render(&self) -> String {
        self._render(self.start())
    }

    fn _render(&self, id: usize) -> String {
        if let Some(node) = self.get(id) {
            match node {
                &Node::Assign(lhs, rhs) => {
                    format!("{} = {}", self._render(lhs), self._render(rhs))
                }
                &Node::Add(lhs, rhs) => format!("{} + {}", self._render(lhs), self._render(rhs)),
                &Node::Sub(lhs, rhs) => format!("{} - {}", self._render(lhs), self._render(rhs)),
                &Node::Mul(lhs, rhs) => format!("{} * {}", self._render(lhs), self._render(rhs)),
                &Node::Div(lhs, rhs) => format!("{} / {}", self._render(lhs), self._render(rhs)),
                &Node::Exp(lhs, rhs) => format!("{} ^ {}", self._render(lhs), self._render(rhs)),
                &Node::Neg(arg) => format!("-{}", self._render(arg)),
                &Node::Adv(arg) => format!("{}a", self._render(arg)),
                &Node::DisAdv(arg) => format!("{}d", self._render(arg)),
                &Node::Sort(arg) => format!("{}s", self._render(arg)),
                &Node::Keep(lhs, rhs) => format!("{}k{}", self._render(lhs), self._render(rhs)),
                &Node::Roll(q, d) => {
                    if q == 1 {
                        format!("d{d}")
                    } else {
                        format!("{q}d{d}")
                    }
                }
                &Node::Natural(n) => n.to_string(),
                Node::Identifier(name) => name.clone(),
                Node::Call(name, args) => {
                    format!(
                        "{name}({})",
                        args.iter().fold(String::new(), |mut acc, el| {
                            if !acc.is_empty() {
                                acc.push_str(", ");
                            }
                            acc.push_str(&self._render(*el));
                            acc
                        })
                    )
                }
            }
        } else {
            "ERROR".to_string()
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
    Assign,
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
            Operator::Assign => 1,
            Operator::Add => 2,
            Operator::Sub => 2,
            Operator::Mul => 3,
            Operator::Div => 3,
            Operator::Exp => 5,
            Operator::Neg => 4,
            Operator::Keep => 6,
            Operator::Adv => 4,
            Operator::DisAdv => 4,
            Operator::Sort => 4,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => false,
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
            Operator::Assign => true,
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

    fn binary(self, lhs: usize, rhs: usize) -> ParseResult<Node> {
        match self {
            Operator::Assign => Ok(Node::Assign(lhs, rhs)),
            Operator::Add => Ok(Node::Add(lhs, rhs)),
            Operator::Sub => Ok(Node::Sub(lhs, rhs)),
            Operator::Mul => Ok(Node::Mul(lhs, rhs)),
            Operator::Div => Ok(Node::Div(lhs, rhs)),
            Operator::Exp => Ok(Node::Exp(lhs, rhs)),
            Operator::Keep => Ok(Node::Keep(lhs, rhs)),
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

    fn unary(self, operand: usize) -> ParseResult<Node> {
        match self {
            Operator::Neg => Ok(Node::Neg(operand)),
            Operator::Adv => Ok(Node::Adv(operand)),
            Operator::DisAdv => Ok(Node::DisAdv(operand)),
            Operator::Sort => Ok(Node::Sort(operand)),
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
            Token::Comma => err(", is not an operator."),
            Token::Assign => Ok(Self::Assign),
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

    fn expr(&mut self) -> ParseResult<usize> {
        let mut id = self.term()?;

        while let Some(Ok(op)) = self.peek().map(Operator::from)
            && op.is_binary()
        {
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

    fn term(&mut self) -> ParseResult<usize> {
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
            Token::Assign => err("= unexpected."),
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
        }?;

        while let Some(Ok(op)) = self.peek().map(Operator::from)
            && op.is_unary_postfix()
        {
            self.push_operator(op);
            self.next()?; // throw away token
        }

        Ok(id)
    }

    fn call(&mut self, name: String) -> ParseResult<usize> {
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
        Ok(self.push_operand(Node::Call(name, args)))
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

    fn pop_operator(&mut self) -> ParseResult<usize> {
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

pub fn lex(input: &[Token]) -> Result<Ast, String> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod test {
    use crate::roll::{parse, token::tokenise};

    use super::*;

    fn exprs(ast: &Ast) -> &[Node] {
        &ast.0
    }

    fn check_exprs(input: &str, expected: Vec<Node>) {
        assert_eq!(exprs(&lex(&tokenise(input).unwrap()).unwrap()), expected)
    }

    fn root(ast: &Ast) -> Option<&Node> {
        ast.get(ast.start())
    }

    #[test]
    fn test_parse_addition() {
        let ast = lex(&[Token::Natural(2), Token::Plus, Token::Natural(3)]).unwrap();
        assert_eq!(
            exprs(&ast),
            vec![Node::Natural(2), Node::Natural(3), Node::Add(0, 1)]
        );
        assert_eq!(root(&ast), Some(&Node::Add(0, 1)));
    }

    #[test]
    fn test_negation() {
        let ast = lex(&[Token::Minus, Token::Natural(3)]).unwrap();
        assert_eq!(exprs(&ast), vec![Node::Natural(3), Node::Neg(0)]);
        assert_eq!(root(&ast), Some(&Node::Neg(0)));

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
        let ast = lex(&[
            Token::Minus,
            Token::Natural(2),
            Token::Exp,
            Token::Natural(3),
        ])
        .unwrap();

        assert_eq!(root(&ast), Some(&Node::Neg(2)));
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
                Node::Natural(3),
                Node::Natural(4),
                Node::Sub(0, 1),
                Node::Natural(5),
                Node::Add(2, 3)
            ]
        );
    }

    #[test]
    fn test_binary_operators() {
        let token = Some(&Token::Plus);
        if let Some(Ok(op)) = token.map(Operator::from)
            && op.is_binary()
        {
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
            vec![Node::Roll(10, 8), Node::Natural(8), Node::Keep(0, 1)]
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
    fn test_render() {
        let src = "func() = var = other * 3 - 1";
        assert_eq!(parse(src).unwrap().0.render(), src)
    }
}
