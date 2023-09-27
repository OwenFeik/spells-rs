mod expr;
mod token;

pub fn parse(input: &str) -> Result<expr::Expr, String> {
    expr::lex(&token::tokenise(input))
}

#[cfg(test)]
mod test {
    use super::expr::Expr;
    use super::*;

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse("4 + 3 - 2 * 5").unwrap(),
            Expr::Sub(
                Box::new(Expr::Add(
                    Box::new(Expr::Natural(4)),
                    Box::new(Expr::Natural(3))
                )),
                Box::new(Expr::Mul(
                    Box::new(Expr::Natural(2)),
                    Box::new(Expr::Natural(5))
                ))
            )
        );
    }

    #[test]
    fn test_parse_exponent() {
        assert_eq!(
            parse("-5^3*3").unwrap(),
            expr::Expr::Mul(
                Box::new(Expr::Neg(Box::new(Expr::Exp(
                    Box::new(Expr::Natural(5)),
                    Box::new(Expr::Natural(3))
                )))),
                Box::new(Expr::Natural(3))
            )
        );
    }
}
