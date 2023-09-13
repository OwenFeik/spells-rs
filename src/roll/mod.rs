
#[derive(Debug)]
enum Token {
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
    Sort
}

impl Token {
    fn from(c: char) -> Option<Self> {
        match c {
            _ if c.is_numeric() => c.to_digit(10).map(Self::Natural),
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
            _ => None
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
            match self {
                Self::Natural(v) => return (None, Some(Self::Roll(v, 0))),
                _ => {}
            }
        }

        (Some(self), Self::from(c))
    }
}

#[derive(Debug)]
pub struct Roll(Vec<Token>);

pub fn tokenise(input: &str) -> Roll {
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

    Roll(tokens)
}
