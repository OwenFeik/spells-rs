use super::token::Token;

/// Grammar
/// expr := term addsub | term
/// addsub := + expr addsub | - expr addsub | eps
/// T := T * T | T / F | T ^ F | F
/// F := Ra | Rd | Rs | RkN | R | N | (E)
/// R := NdN | dN
/// N := NN | [0-9]
///

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Exp(Box<Expr>, Box<Expr>),
    Adv(Box<Expr>),
    DisAdv(Box<Expr>),
    Sort(Box<Expr>),
    Keep(Box<Expr>),
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
}

fn head(input: &[Token]) -> Option<&Token> {
    input.first()
}

fn tail(input: &[Token]) -> &[Token] {
    if input.len() < 2 {
        &[]
    } else {
        &input[1..]
    }
}

fn parts(input: &[Token]) -> (Option<&Token>, &[Token]) {
    (head(input), tail(input))
}

fn eat(input: &[Token], token: Token) -> Option<&[Token]> {
    let (head, tail) = parts(input);
    if token == *head? {
        Some(tail)
    } else {
        None
    }
}

type ParseResult<'a> = Option<(Expr, &'a [Token])>;

fn parens(input: &[Token]) -> ParseResult {
    if let (Some(Token::ParenOpen), rest) = parts(input) {
        let (expr, rest) = expr(rest)?;
        if let (Some(Token::ParenClose), rest) = parts(rest) {
            return Some((expr, rest));
        }
    }
    None
}

fn addsub_inner(lhs: Expr, input: &[Token]) -> ParseResult {
    if input.is_empty() {
        Some((lhs, input))
    } else if let Some(rest) = eat(input, Token::Plus) {
        let (rhs, rest) = expr(rest)?;
        addsub_inner(Expr::add(lhs, rhs), rest)
    } else if let Some(rest) = eat(input, Token::Minus) {
        let (rhs, rest) = expr(rest)?;
        addsub_inner(Expr::sub(lhs, rhs), rest)
    } else {
        None
    }
}

fn addsub(input: &[Token]) -> ParseResult {
    let (lhs, rest) = term(input)?;
    addsub_inner(lhs, rest)
}

fn expr(input: &[Token]) -> ParseResult {
    addsub(input).or_else(|| term(input))
}

fn roll(input: &[Token]) -> ParseResult {
    if let &Token::Roll(q, d) = head(input)? {
        Some((Expr::Roll(q, d), tail(input)))
    } else {
        None
    }
}

fn natural(input: &[Token]) -> ParseResult {
    if let &Token::Natural(n) = head(input)? {
        Some((Expr::Natural(n), tail(input)))
    } else {
        None
    }
}

fn term(input: &[Token]) -> ParseResult {
    roll(input)
        .or_else(|| natural(input))
        .or_else(|| parens(input))
}

fn parse(input: &[Token]) -> ParseResult {
    expr(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_addition() {
        let (expr, rest) = parse(&[Token::Natural(2), Token::Plus, Token::Natural(3)]).unwrap();
        assert_eq!(rest, Vec::new());
        assert_eq!(
            expr,
            Expr::Add(Box::new(Expr::Natural(2)), Box::new(Expr::Natural(3)))
        );
    }

    #[test]
    fn test_parse_repeated_addition() {
        let (expr, rest) = parse(&[
            Token::Natural(2),
            Token::Plus,
            Token::Natural(3),
            Token::Plus,
            Token::Natural(4),
        ])
        .unwrap();
        assert_eq!(rest, Vec::new());

        let rhs = Expr::Add(Box::new(Expr::Natural(3)), Box::new(Expr::Natural(4)));
        assert_eq!(expr, Expr::Add(Box::new(Expr::Natural(2)), Box::new(rhs)));
    }

    #[test]
    fn test_addition_subtraction() {
        let (expr, rest) = parse(&[
            Token::Natural(3),
            Token::Minus,
            Token::Natural(4),
            Token::Plus,
            Token::Natural(5),
        ])
        .unwrap();
        assert_eq!(rest, Vec::new());
        assert_eq!(
            expr,
            Expr::Add(
                Box::new(Expr::Sub(
                    Box::new(Expr::Natural(3)),
                    Box::new(Expr::Natural(4))
                )),
                Box::new(Expr::Natural(5)),
            )
        )
    }
}
