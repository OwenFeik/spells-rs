use super::token::Token;

/// Grammar
/// E := (E) | T + E | T - E | T
/// T := T * F | T / F | T ^ F | F
/// F := Ra | Rd | Rs | RkN | R | N 
/// R := NdN | dN
/// N := NN | [0-9]

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
    &input[1..]
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
    if let (Some(Token::Plus), rest) = parts(rest) {
        let (expr, rest) = expr(rest)?;
        Some((Expr::Sub(Box::new(term), Box::new(expr)), rest))
    } else {
        None
    }
}

fn expr(input: &[Token]) -> ParseResult {
    parens(input).or_else(|| add(input)).or_else(|| sub(input)).or_else(|| term(input))
}

fn mul(input: &[Token]) -> ParseResult {
    
}

fn term(input: &[Token]) -> ParseResult {

}

fn factor(input: &[Token]) -> ParseResult {

}

fn parse(input: &[Token]) -> Option<(&[Token], Expr)> {
    let Some(first) = input.first() else {
        return None;
    };

    let rest = tail(input);
    match first {
        &Token::Natural(n) => Some((rest, Expr::Natural(n))),
        &Token::Roll(q, d) => Some((rest, Expr::Roll(q, d))),
        &Token::ParenOpen => {
            let (rest, expr) = parse(rest)?;
            if matches!(rest.first(), Some(Token::ParenClose)) {
                Some((tail(rest), expr))
            } else {
                None
            }
        },
        &Token::ParenClose => None,
        &Token::Plus => todo!(),
        &Token::Minus => todo!(),
        &Token::Times => todo!(),
        &Token::Divide => todo!(),
        &Token::Exp => todo!(),
        &Token::Keep => todo!(),
        &Token::Advantage => todo!(),
        &Token::Disadvantage => todo!(),
        &Token::Sort => todo!(),
        
    }
}
