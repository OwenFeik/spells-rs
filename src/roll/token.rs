#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Natural(u32),
    Roll(u32, u32),
    ParenOpen,
    ParenClose,
    Plus,
    Minus,
    Times,
    Divide,
    Exp,
    Keep,
    Advantage,
    Disadvantage,
    Sort,
    Identifier(String),
}

impl Token {
    fn from(c: char) -> Option<Self> {
        match c {
            '(' => Some(Self::ParenOpen),
            ')' => Some(Self::ParenClose),
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

    fn consume(self, c: char) -> (Option<Self>, Option<Self>) {
        if let Some(n) = c.to_digit(10) {
            match self {
                Self::Natural(v) => return (None, Some(Self::Natural(v * 10 + n))),
                Self::Roll(q, s) => return (None, Some(Self::Roll(q, s * 10 + n))),
                Self::Disadvantage => return (None, Some(Self::Roll(1, n))),
                _ => {}
            }
        }

        if c == 'd' {
            if let Self::Natural(v) = self {
                return (None, Some(Self::Roll(v, 0)));
            }
        }

        (Some(self), Self::from(c))
    }
}

pub fn tokenise(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut current: Option<Token> = None;
    for c in input.chars() {
        if let Some(token) = current {
            let finished;
            (finished, current) = token.consume(c);
            if let Some(finished) = finished {
                tokens.push(finished);
            }
        } else {
            current = Token::from(c);
        }
    }

    if let Some(current) = current {
        if let (Some(token), _) = current.consume(' ') {
            tokens.push(token);
        }
    }

    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tokenise_roll() {
        assert_eq!(tokenise("1d4"), vec![Token::Roll(1, 4)]);
        assert_eq!(tokenise("d4"), vec![Token::Roll(1, 4)]);
        assert_eq!(tokenise("8d8"), vec![Token::Roll(8, 8)]);
        assert_eq!(tokenise("d20"), vec![Token::Roll(1, 20)]);
        assert_eq!(
            tokenise("d20 d20"),
            vec![Token::Roll(1, 20), Token::Roll(1, 20)]
        );
    }

    #[test]
    fn test_tokenise_ops() {
        assert_eq!(
            tokenise("+ - * / ^"),
            vec![
                Token::Plus,
                Token::Minus,
                Token::Times,
                Token::Divide,
                Token::Exp
            ]
        );
        assert_eq!(
            tokenise("a d k s"),
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
            tokenise("(d4 * 3) + (8d8k5)"),
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
}
