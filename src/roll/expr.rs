use super::token::Token;

/// Grammar
/// E := T + E | T - E | T
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

fn add(input: &[Token]) -> ParseResult {
    let (term, rest) = term(input)?;
    if let (Some(Token::Plus), rest) = parts(rest) {
        let (expr, rest) = expr(rest)?;
        Some((Expr::Add(Box::new(term), Box::new(expr)), rest))
    } else {
        None
    }
}

fn sub(input: &[Token]) -> ParseResult {
    let (term, rest) = term(input)?;
    if let (Some(Token::Minus), rest) = parts(rest) {
        let (expr, rest) = expr(rest)?;
        Some((Expr::Sub(Box::new(term), Box::new(expr)), rest))
    } else {
        None
    }
}

fn expr(input: &[Token]) -> ParseResult {
    add(input).or_else(|| sub(input)).or_else(|| term(input))
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
                Box::new(Expr::Natural(3)),
            )
        )
    }
}
