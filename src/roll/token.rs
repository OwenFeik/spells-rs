#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Natural(u32),
    Roll(u32, u32),
    ParenOpen,
    ParenClose,
    Comma,
    Assign,
    Define,
    Plus,
    Minus,
    Times,
    Divide,
    Exp,
    Keep,
    Advantage,
    Disadvantage,
    Sort,
}

impl Token {
    fn from(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::ParenOpen),
            ')' => Some(Self::ParenClose),
            ',' => Some(Self::Comma),
            '=' => Some(Self::Assign),
            ':' => Some(Self::Define),
            '+' => Some(Self::Plus),
            '-' => Some(Self::Minus),
            '*' => Some(Self::Times),
            '/' => Some(Self::Divide),
            '^' => Some(Self::Exp),
            'k' => Some(Self::Keep),
            'a' => Some(Self::Advantage),
            'd' => Some(Self::Disadvantage),
            's' => Some(Self::Sort),
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
            Token::Assign => '=',
            Token::Define => ':',
            Token::Plus => '+',
            Token::Minus => '-',
            Token::Times => '*',
            Token::Divide => '/',
            Token::Exp => '^',
            Token::Keep => 'k',
            Token::Advantage => 'a',
            Token::Disadvantage => 'd',
            Token::Sort => 's',
        }
    }

    fn consume(self, c: char) -> Result<(Option<Self>, Option<Self>), String> {
        if let Some(n) = c.to_digit(10) {
            match self {
                Self::Identifier(mut name) => {
                    name.push(c);
                    return Ok((None, Some(Self::Identifier(name))));
                }
                Self::Natural(v) => return Ok((None, Some(Self::Natural(v * 10 + n)))),
                Self::Roll(q, s) => return Ok((None, Some(Self::Roll(q, s * 10 + n)))),
                Self::Disadvantage => return Ok((None, Some(Self::Roll(1, n)))),
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
                _ if self.char().is_alphabetic() => {
                    let name = [self.char(), c].iter().collect();
                    return Ok((None, Some(Self::Identifier(name))));
                }
                _ => {}
            }
        }

        if matches!(self, Self::Define) {
            return if c == '=' {
                Ok((Some(self), None)) // Define (:=) finished.
            } else {
                Err(format!("Unexpected character following \":\": {c}"))
            };
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
                Token::Plus,
                Token::Minus,
                Token::Times,
                Token::Divide,
                Token::Exp
            ]
        );
        assert_eq!(
            tok_unwrap("a d k s"),
            vec![
                Token::Advantage,
                Token::Disadvantage,
                Token::Keep,
                Token::Sort
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
                Token::Times,
                Token::Natural(3),
                Token::ParenClose,
                Token::Plus,
                Token::ParenOpen,
                Token::Roll(8, 8),
                Token::Keep,
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
                Token::Plus,
                Token::Identifier("PROF".to_string()),
                Token::Plus,
                Token::Identifier("STR".to_string()),
                Token::Plus,
                Token::Natural(1)
            ]
        )
    }

    #[test]
    fn test_tokenise_ops_identifiers() {
        assert_eq!(
            tok_unwrap("dword d aword a"),
            vec![
                Token::Identifier("dword".to_string()),
                Token::Disadvantage,
                Token::Identifier("aword".to_string()),
                Token::Advantage,
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
                Token::Plus,
                Token::Natural(2),
                Token::Comma,
                Token::Identifier("arg2".into()),
                Token::Comma,
                Token::ParenOpen,
                Token::Natural(2),
                Token::Exp,
                Token::Natural(3),
                Token::ParenClose,
                Token::ParenClose,
            ]
        )
    }

    #[test]
    fn test_tokenise_assign_define() {
        assert_eq!(
            tok_unwrap("fn := var = 2"),
            vec![
                Token::Identifier("fn".into()),
                Token::Define,
                Token::Identifier("var".into()),
                Token::Assign,
                Token::Natural(2)
            ]
        )
    }
}
