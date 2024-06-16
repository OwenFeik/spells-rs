use crate::{operator::Operator, Res};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Natural(u64),
    Decimal(String),
    Roll(u64, u64),
    Operator(Operator),
    String(String),
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    Comma,
}

impl Token {
    fn from(c: char) -> Option<Self> {
        match c {
            ',' => Some(Self::Comma),
            '(' => Some(Self::ParenOpen),
            ')' => Some(Self::ParenClose),
            '[' => Some(Self::BracketOpen),
            ']' => Some(Self::BracketClose),
            '"' => Some(Self::String(String::new())),
            '_' => Some(Self::Identifier(String::from("_"))),
            '.' => Some(Self::Decimal(String::from("."))),
            _ if c.is_numeric() => c.to_digit(10).map(|v| Self::Natural(v as u64)),
            _ if c.is_alphabetic() => Some(Self::Identifier(String::from(c))),
            _ => {
                for op in Operator::TOKENS {
                    if op.str().starts_with(c) {
                        return Some(Self::Operator(*op));
                    }
                }
                None
            }
        }
    }

    fn string(&self) -> String {
        match self {
            Token::Identifier(name) => name.clone(),
            Token::Natural(num) => num.to_string(),
            Token::Decimal(num) => num.to_string(),
            Token::Roll(q, d) => format!("{q}d{d}"),
            Token::String(s) => format!("\"{}\"", s.clone()),
            Token::ParenOpen => String::from("("),
            Token::ParenClose => String::from(")"),
            Token::BracketOpen => String::from("["),
            Token::BracketClose => String::from("]"),
            Token::Comma => String::from(","),
            Token::Operator(op) => op.str().to_owned(),
        }
    }

    fn consume(self, c: char) -> Res<(Option<Self>, Option<Self>)> {
        if let Token::String(mut val) = self {
            return match c {
                '"' => {
                    if val.ends_with('\\') {
                        val.truncate(val.len() - 1);
                        val.push('"');
                        extended(Self::String(val))
                    } else {
                        finished(Self::String(val))
                    }
                }
                _ => {
                    val.push(c);
                    extended(Self::String(val))
                }
            };
        }

        if let Some(n) = c.to_digit(10) {
            let n = n as u64;
            match self {
                Self::Identifier(d) if d == "d" => {
                    return extended(Self::Roll(1, n));
                }
                Self::Identifier(mut name) => {
                    name.push(c);
                    return extended(Self::Identifier(name));
                }
                Self::Natural(v) => return extended(Self::Natural(v * 10 + n)),
                Self::Roll(q, s) => return extended(Self::Roll(q, s * 10 + n)),
                Self::Operator(Operator::DisAdv) => return extended(Self::Roll(1, n)),
                Self::Decimal(mut text) => {
                    text.push(c);
                    return extended(Self::Decimal(text));
                }
                _ => {}
            }
        }

        if c.is_alphabetic() || c == '_' {
            match self {
                Self::Natural(v) if c == 'd' => return Ok((None, Some(Self::Roll(v, 0)))),
                Self::Identifier(mut name) => {
                    name.push(c);
                    return extended(Self::Identifier(name));
                }
                Self::Roll(..) if c == 'a' || c == 'd' || c == 'k' || c == 's' => {
                    return match c {
                        'a' => finished_and(self, Self::Operator(Operator::Adv)),
                        'd' => finished_and(self, Self::Operator(Operator::DisAdv)),
                        'k' => finished_and(self, Self::Operator(Operator::Keep)),
                        _ => finished_and(self, Self::Operator(Operator::Sort)),
                    }
                }
                _ if self.string().chars().all(char::is_alphanumeric) => {
                    return extended(Self::Identifier(format!("{}{}", self.string(), c)));
                }
                _ => {}
            }
        }

        if c == '.' {
            return match self {
                Self::Natural(v) => extended(Self::Decimal(format!("{v}."))),
                _ => finished_and(self, Self::Decimal(".".to_string())),
            };
        }

        if c == '=' {
            match self {
                Self::Operator(Operator::Assign) => {
                    return finished(Token::Operator(Operator::Equal));
                }
                Self::Operator(Operator::GreaterThan) => {
                    return finished(Token::Operator(Operator::GreaterEqual));
                }
                Self::Operator(Operator::LessThan) => {
                    return finished(Token::Operator(Operator::LessEqual));
                }
                _ => {}
            }
        }

        Ok((Some(self), Self::from(c)))
    }
}

fn extended(token: Token) -> Res<(Option<Token>, Option<Token>)> {
    Ok((None, Some(token)))
}

fn finished(token: Token) -> Res<(Option<Token>, Option<Token>)> {
    Ok((Some(token), None))
}

fn finished_and(completed: Token, next: Token) -> Res<(Option<Token>, Option<Token>)> {
    Ok((Some(completed), Some(next)))
}

pub fn tokenise(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    let mut current: Option<Token> = None;
    for c in input.chars() {
        if let Some(token) = current {
            let finished;
            (finished, current) = token.consume(c)?;
            if let Some(finished) = finished {
                tokens.push(finished);
            }
        } else {
            current = Token::from(c);
        }
    }

    if let Some(current) = current {
        if let (Some(token), _) = current.consume(' ')? {
            tokens.push(token);
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    fn tok_unwrap(input: &str) -> Vec<Token> {
        tokenise(input).unwrap()
    }

    #[test]
    fn test_tokenise_roll() {
        assert_eq!(tok_unwrap("1d4"), vec![Token::Roll(1, 4)]);
        assert_eq!(tok_unwrap("d4"), vec![Token::Roll(1, 4)]);
        assert_eq!(tok_unwrap("8d8"), vec![Token::Roll(8, 8)]);
        assert_eq!(tok_unwrap("d20"), vec![Token::Roll(1, 20)]);
        assert_eq!(
            tok_unwrap("d20 d20"),
            vec![Token::Roll(1, 20), Token::Roll(1, 20)]
        );
    }

    #[test]
    fn test_tokenise_ops() {
        assert_eq!(
            tok_unwrap("+ - * / ^"),
            vec![
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Sub),
                Token::Operator(Operator::Mul),
                Token::Operator(Operator::Div),
                Token::Operator(Operator::Exp)
            ]
        );
        assert_eq!(
            tok_unwrap("d4a d8d 4d6k4 d8s"),
            vec![
                Token::Roll(1, 4),
                Token::Operator(Operator::Adv),
                Token::Roll(1, 8),
                Token::Operator(Operator::DisAdv),
                Token::Roll(4, 6),
                Token::Operator(Operator::Keep),
                Token::Natural(4),
                Token::Roll(1, 8),
                Token::Operator(Operator::Sort)
            ]
        );
    }

    #[test]
    fn test_tokenise_exprs() {
        assert_eq!(
            tok_unwrap("(d4 * 3) + (8d8k5)"),
            vec![
                Token::ParenOpen,
                Token::Roll(1, 4),
                Token::Operator(Operator::Mul),
                Token::Natural(3),
                Token::ParenClose,
                Token::Operator(Operator::Add),
                Token::ParenOpen,
                Token::Roll(8, 8),
                Token::Operator(Operator::Keep),
                Token::Natural(5),
                Token::ParenClose
            ]
        )
    }

    #[test]
    fn test_tokenise_identifiers() {
        assert_eq!(
            tok_unwrap("d20 + PROF + STR + 1"),
            vec![
                Token::Roll(1, 20),
                Token::Operator(Operator::Add),
                Token::Identifier("PROF".to_string()),
                Token::Operator(Operator::Add),
                Token::Identifier("STR".to_string()),
                Token::Operator(Operator::Add),
                Token::Natural(1)
            ]
        )
    }

    #[test]
    fn test_tokenise_ops_identifiers() {
        assert_eq!(
            tok_unwrap("d20d dword d aword a d20a"),
            vec![
                Token::Roll(1, 20),
                Token::Operator(Operator::DisAdv),
                Token::Identifier("dword".to_string()),
                Token::Identifier("d".to_string()),
                Token::Identifier("aword".to_string()),
                Token::Identifier("a".to_string()),
                Token::Roll(1, 20),
                Token::Operator(Operator::Adv),
            ]
        )
    }

    #[test]
    fn test_tokenise_call() {
        assert_eq!(
            tok_unwrap("function(arg1, 3 + 2, arg2, (2 ^ 3))"),
            vec![
                Token::Identifier("function".into()),
                Token::ParenOpen,
                Token::Identifier("arg1".into()),
                Token::Comma,
                Token::Natural(3),
                Token::Operator(Operator::Add),
                Token::Natural(2),
                Token::Comma,
                Token::Identifier("arg2".into()),
                Token::Comma,
                Token::ParenOpen,
                Token::Natural(2),
                Token::Operator(Operator::Exp),
                Token::Natural(3),
                Token::ParenClose,
                Token::ParenClose,
            ]
        )
    }

    #[test]
    fn test_tokenise_assign_define() {
        assert_eq!(
            tok_unwrap("fn() = var = 2"),
            vec![
                Token::Identifier("fn".into()),
                Token::ParenOpen,
                Token::ParenClose,
                Token::Operator(Operator::Assign),
                Token::Identifier("var".into()),
                Token::Operator(Operator::Assign),
                Token::Natural(2)
            ]
        )
    }

    #[test]
    fn test_tokenise_underscore_identifier() {
        assert_eq!(
            tok_unwrap("underscore_name"),
            vec![Token::Identifier("underscore_name".into())],
        )
    }

    #[test]
    fn test_tokenise_string() {
        assert_eq!(
            tok_unwrap(r#"var = "string1" + "string2""#),
            vec![
                Token::Identifier("var".into()),
                Token::Operator(Operator::Assign),
                Token::String("string1".into()),
                Token::Operator(Operator::Add),
                Token::String("string2".into())
            ]
        )
    }

    #[test]
    fn test_tokenise_decimal() {
        assert_eq!(
            tok_unwrap("3.14159"),
            vec![Token::Decimal("3.14159".into())]
        )
    }

    #[test]
    fn test_tokenise_decimal_call() {
        assert_eq!(
            tok_unwrap("floor(2.72)"),
            vec![
                Token::Identifier("floor".into()),
                Token::ParenOpen,
                Token::Decimal("2.72".into()),
                Token::ParenClose
            ]
        )
    }

    #[test]
    fn test_tokenise_all_ops() {
        for op in Operator::TOKENS {
            assert_eq!(tok_unwrap(op.str()), vec![Token::Operator(*op)]);
        }
    }
}
