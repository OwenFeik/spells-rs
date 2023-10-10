#[cfg(no)]
mod eval;
mod expr;
mod token;

pub fn parse(input: &str) -> Result<expr::Ast, String> {
    expr::lex(&token::tokenise(input))
}

#[cfg(test)]
mod test {
    use super::expr::Expr;
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse("4 + 3 - 2 * 5").unwrap().exprs(),
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
            parse("-5^3*3").unwrap().exprs(),
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
