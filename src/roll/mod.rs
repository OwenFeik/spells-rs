mod expr;
mod token;

#[cfg(test)]
mod test {
    use super::expr::Expr;
    use super::{expr, token};

    #[test]
    fn test_parse_string() {
        assert_eq!(
            expr::parse(&token::tokenise("4 + 3 - 2 * 5")).unwrap(),
            expr::Expr::Sub(
                Box::new(Expr::Add(
                    Box::new(Expr::Natural(4)),
                    Box::new(Expr::Natural(3))
                )),
                Box::new(Expr::Mul(
                    Box::new(Expr::Natural(2)),
                    Box::new(Expr::Natural(5))
                ))
            )
        )
    }
}
