use crate::operator::Operator;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Natural(u32),
    Roll(u32, u32),
    Operator(Operator),
    ParenOpen,
    ParenClose,
    Comma,
}

impl Token {
    fn from(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::ParenOpen),
            ')' => Some(Self::ParenClose),
            ',' => Some(Self::Comma),
            '=' => Some(Self::Operator(Operator::Assign)),
            '+' => Some(Self::Operator(Operator::Add)),
            '-' => Some(Self::Operator(Operator::Sub)),
            '*' => Some(Self::Operator(Operator::Mul)),
            '/' => Some(Self::Operator(Operator::Div)),
            '^' => Some(Self::Operator(Operator::Exp)),
            'k' => Some(Self::Operator(Operator::Keep)),
            '_' => Some(Self::Identifier(String::from("_"))),
            _ if c.is_numeric() => c.to_digit(10).map(Self::Natural),
            _ if c.is_alphabetic() => Some(Self::Identifier(String::from(c))),
            _ => None,
        }
    }

    fn char(&self) -> char {
        match self {
            Token::Identifier(_) => '!',
            Token::Natural(_) => '#',
            Token::Roll(_, _) => '%',
            Token::ParenOpen => '(',
            Token::ParenClose => ')',
            Token::Comma => ',',
            Token::Operator(op) => op.char(),
        }
    }

    fn consume(self, c: char) -> Result<(Option<Self>, Option<Self>), String> {
        if let Some(n) = c.to_digit(10) {
            match self {
                Self::Identifier(d) if d == "d" => {
                    return Ok((None, Some(Self::Roll(1, n))));
                }
                Self::Identifier(mut name) => {
                    name.push(c);
                    return Ok((None, Some(Self::Identifier(name))));
                }
                Self::Natural(v) => return Ok((None, Some(Self::Natural(v * 10 + n)))),
                Self::Roll(q, s) => return Ok((None, Some(Self::Roll(q, s * 10 + n)))),
                Self::Operator(Operator::DisAdv) => return Ok((None, Some(Self::Roll(1, n)))),
                _ => {}
            }
        }

        if c.is_alphabetic() {
            match self {
                Self::Natural(v) if c == 'd' => return Ok((None, Some(Self::Roll(v, 0)))),
                Self::Identifier(mut name) => {
                    name.push(c);
                    return Ok((None, Some(Self::Identifier(name))));
                }
                Self::Roll(..) if c == 'a' || c == 'd' || c == 's' => {
                    return match c {
                        'a' => Ok((Some(self), Some(Token::Operator(Operator::Adv)))),
                        'd' => Ok((Some(self), Some(Token::Operator(Operator::DisAdv)))),
                        _ => Ok((Some(self), Some(Token::Operator(Operator::Sort)))),
                    }
                }
                _ if self.char().is_alphabetic() => {
                    let name = [self.char(), c].iter().collect();
                    return Ok((None, Some(Self::Identifier(name))));
                }
                _ => {}
            }
        }

        Ok((Some(self), Self::from(c)))
    }
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
            tok_unwrap("d4a d8d k 8d8s"),
            vec![
                Token::Roll(1, 4),
                Token::Operator(Operator::Adv),
                Token::Roll(1, 8),
                Token::Operator(Operator::DisAdv),
                Token::Operator(Operator::Keep),
                Token::Roll(8, 8),
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
}
