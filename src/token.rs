use crate::{err, operator::Operator, Res};

const COMMENT: char = '#';

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Identifier(String),
    Natural(u64),
    Decimal(f64),
    Roll(u64, u64),
    Operator(Operator),
    String(String),
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    Comma,
}

impl Tok {
    pub fn identifier<S: ToString>(identifier: S) -> Self {
        Self::Identifier(identifier.to_string())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    tok: Tok,
    line: usize,
    col: usize,
    len: usize,
}

impl Token {
    pub fn new(tok: Tok, line: usize, col: usize, len: usize) -> Self {
        Self {
            tok,
            line,
            col,
            len,
        }
    }

    pub fn inner(&self) -> &Tok {
        &self.tok
    }
}

fn read_string(input: &[char]) -> Res<(usize, Tok)> {
    debug_assert!(input[0] == '"');

    let mut s = String::new();
    let mut escaped = false;
    let mut i = 1; // Skip opening quote.
    while let Some(c) = input.get(i).copied() {
        i = i + 1;
        match c {
            '\\' => {
                if escaped {
                    s.push('\\');
                    escaped = false;
                } else {
                    escaped = true;
                }
            }
            '"' => {
                if escaped {
                    s.push('"');
                    escaped = false;
                } else {
                    return Ok((i, Tok::String(s)));
                }
            }
            'n' if escaped => {
                s.push('\n');
                escaped = false;
            }
            't' if escaped => {
                s.push('\t');
                escaped = false;
            }
            '\n' => {
                return err("Strings must be single line.");
            }
            _ => s.push(c),
        }
    }
    return err("Unterminated string.");
}

fn read_number(input: &[char]) -> Res<(usize, Tok)> {
    debug_assert!(input[0].is_numeric() || input[0] == 'd');

    let mut s = String::new();
    let mut is_decimal = false;
    let mut is_roll = false;
    let mut i = 0;
    while let Some(c) = input.get(i).copied() {
        i = i + 1;
        match c {
            '.' => {
                s.push('.');
                if is_decimal || is_roll {
                    return Err(format!("Invalid literal: {s}"));
                } else {
                    is_decimal = true;
                }
            }
            'd' => {
                if is_roll {
                    break;
                } else if is_decimal {
                    return Err(format!("Invalid literal: {s}"));
                } else {
                    s.push('d');
                    is_roll = true;
                }
            }
            _ if c.is_numeric() => {
                s.push(c);
            }
            _ => break,
        }
    }

    let tok = if is_decimal {
        Tok::Decimal(s.parse::<f64>().map_err(|e| e.to_string())?)
    } else if is_roll {
        if let Some((q, d)) = s.split_once('d') {
            let q = if q.is_empty() {
                1
            } else {
                q.parse::<u64>().map_err(|e| e.to_string())?
            };
            let d = d.parse::<u64>().map_err(|e| e.to_string())?;
            Tok::Roll(q, d)
        } else {
            return Err(format!("Failed to parse roll literal: {s}"));
        }
    } else {
        Tok::Natural(s.parse::<u64>().map_err(|e| e.to_string())?)
    };
    Ok((s.len(), tok))
}

fn read_identifier(input: &[char]) -> Res<(usize, Tok)> {
    debug_assert!(input[0] == '_' || input[0].is_alphabetic());

    let mut s = String::new();
    for &c in input {
        if c == '_' || c.is_alphabetic() || (!s.is_empty() && c.is_numeric()) {
            s.push(c);
        } else {
            break;
        }
    }

    Ok((s.len(), Tok::Identifier(s)))
}

fn read_token(input: &[char]) -> Res<(usize, Tok)> {
    for op in Operator::TOKENS {
        if input.starts_with(op.chars()) {
            return Ok((op.chars().len(), Tok::Operator(*op)));
        }
    }

    match input.get(0) {
        None => err("Input ended unexpectedly."),
        Some(',') => Ok((1, Tok::Comma)),
        Some('(') => Ok((1, Tok::ParenOpen)),
        Some(')') => Ok((1, Tok::ParenClose)),
        Some('[') => Ok((1, Tok::BracketOpen)),
        Some(']') => Ok((1, Tok::BracketClose)),
        Some('"') => read_string(input),
        Some('.') => read_number(input),
        Some(c) if c.is_numeric() => read_number(input),
        Some('_') => read_identifier(input),
        Some('d') => read_number(input).or_else(|_| read_identifier(input)),
        Some(c) if c.is_alphabetic() => read_identifier(input),
        Some(c) => Err(format!("{c} unexpected")),
    }
}

fn read_comment(input: &[char]) -> usize {
    debug_assert!(input[0] == COMMENT);
    let mut len = 0;
    for &c in input {
        len += 1;
        if c == '\n' {
            return len;
        }
    }
    return len;
}

fn maybe_read_postfix_roll_op(input: &[char]) -> Res<(usize, Tok)> {
    let is_operator = if let Some(c) = input.get(1)
        && !c.is_alphabetic()
        && *c != '_'
    {
        true
    } else {
        input.len() == 1
    };

    if is_operator && let Some(c) = input.get(0) {
        for op in Operator::ROLL_SUFFIX_TOKENS {
            if op.chars() == &[*c] {
                return Ok((1, Tok::Operator(*op)));
            }
        }
    }
    read_token(input)
}

pub fn tokenise(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();

    let mut input: &[char] = chars.as_slice();
    let mut tokens: Vec<Token> = Vec::new();
    let mut line = 1;
    let mut col = 1;
    let mut whitespace_since_token = false;
    while !input.is_empty() {
        match input[0] {
            ' ' | '\t' => {
                col += 1;
                input = &input[1..];
                whitespace_since_token = true;
            }
            '\n' => {
                line += 1;
                col = 1;
                input = &input[1..];
                whitespace_since_token = true;
            }
            '#' => {
                let len = read_comment(input);
                line += 1;
                col = 1;
                input = &input[len..];
                whitespace_since_token = true;
            }
            'a' | 'd' | 'k'
                if !whitespace_since_token
                    && let Some(token) = tokens.last()
                    && let Tok::Roll(..) = token.inner() =>
            {
                let (len, tok) = maybe_read_postfix_roll_op(input)?;
                tokens.push(Token::new(tok, line, col, len));
                col += len;
                input = &input[len..];
                whitespace_since_token = false;
            }
            _ => {
                let (len, tok) = read_token(input)?;
                tokens.push(Token::new(tok, line, col, len));
                col += len;
                input = &input[len..];
                whitespace_since_token = false;
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    fn tok_unwrap(input: &str) -> Vec<Tok> {
        tokenise(input)
            .unwrap()
            .into_iter()
            .map(|token| token.tok)
            .collect()
    }

    #[test]
    fn test_tokenise_roll() {
        assert_eq!(tok_unwrap("1d4"), vec![Tok::Roll(1, 4)]);
        assert_eq!(tok_unwrap("d4"), vec![Tok::Roll(1, 4)]);
        assert_eq!(tok_unwrap("8d8"), vec![Tok::Roll(8, 8)]);
        assert_eq!(tok_unwrap("d20"), vec![Tok::Roll(1, 20)]);
        assert_eq!(
            tok_unwrap("d20 d20"),
            vec![Tok::Roll(1, 20), Tok::Roll(1, 20)]
        );
    }

    #[test]
    fn test_tokenise_ops() {
        assert_eq!(
            tok_unwrap("+ - * / ^"),
            vec![
                Tok::Operator(Operator::Add),
                Tok::Operator(Operator::Sub),
                Tok::Operator(Operator::Mul),
                Tok::Operator(Operator::Div),
                Tok::Operator(Operator::Exp)
            ]
        )
    }

    #[test]
    fn test_tokenise_roll_ops() {
        assert_eq!(
            tok_unwrap("d4a d8d 4d6k4"),
            vec![
                Tok::Roll(1, 4),
                Tok::Operator(Operator::Adv),
                Tok::Roll(1, 8),
                Tok::Operator(Operator::DisAdv),
                Tok::Roll(4, 6),
                Tok::Operator(Operator::Keep),
                Tok::Natural(4),
            ]
        )
    }

    #[test]
    fn test_tokenise_roll_suffix() {
        assert_eq!(
            tok_unwrap("d8d d8a 1d8d"),
            vec![
                Tok::Roll(1, 8),
                Tok::Operator(Operator::DisAdv),
                Tok::Roll(1, 8),
                Tok::Operator(Operator::Adv),
                Tok::Roll(1, 8),
                Tok::Operator(Operator::DisAdv)
            ]
        )
    }

    #[test]
    fn test_tokenise_exprs() {
        assert_eq!(
            tok_unwrap("(d4 * 3) + (8d8k5)"),
            vec![
                Tok::ParenOpen,
                Tok::Roll(1, 4),
                Tok::Operator(Operator::Mul),
                Tok::Natural(3),
                Tok::ParenClose,
                Tok::Operator(Operator::Add),
                Tok::ParenOpen,
                Tok::Roll(8, 8),
                Tok::Operator(Operator::Keep),
                Tok::Natural(5),
                Tok::ParenClose
            ]
        )
    }

    #[test]
    fn test_tokenise_identifiers() {
        assert_eq!(
            tok_unwrap("d20 + PROF + STR + 1"),
            vec![
                Tok::Roll(1, 20),
                Tok::Operator(Operator::Add),
                Tok::Identifier("PROF".to_string()),
                Tok::Operator(Operator::Add),
                Tok::Identifier("STR".to_string()),
                Tok::Operator(Operator::Add),
                Tok::Natural(1)
            ]
        )
    }

    #[test]
    fn test_tokenise_ops_identifiers() {
        assert_eq!(
            tok_unwrap("d20d dword d aword a d20a"),
            vec![
                Tok::Roll(1, 20),
                Tok::Operator(Operator::DisAdv),
                Tok::Identifier("dword".to_string()),
                Tok::Identifier("d".to_string()),
                Tok::Identifier("aword".to_string()),
                Tok::Identifier("a".to_string()),
                Tok::Roll(1, 20),
                Tok::Operator(Operator::Adv),
            ]
        )
    }

    #[test]
    fn test_tokenise_call() {
        assert_eq!(
            tok_unwrap("function(arg1, 3 + 2, arg2, (2 ^ 3))"),
            vec![
                Tok::Identifier("function".into()),
                Tok::ParenOpen,
                Tok::Identifier("arg1".into()),
                Tok::Comma,
                Tok::Natural(3),
                Tok::Operator(Operator::Add),
                Tok::Natural(2),
                Tok::Comma,
                Tok::Identifier("arg2".into()),
                Tok::Comma,
                Tok::ParenOpen,
                Tok::Natural(2),
                Tok::Operator(Operator::Exp),
                Tok::Natural(3),
                Tok::ParenClose,
                Tok::ParenClose,
            ]
        )
    }

    #[test]
    fn test_tokenise_assign_define() {
        assert_eq!(
            tok_unwrap("fn() = var = 2"),
            vec![
                Tok::Identifier("fn".into()),
                Tok::ParenOpen,
                Tok::ParenClose,
                Tok::Operator(Operator::Assign),
                Tok::Identifier("var".into()),
                Tok::Operator(Operator::Assign),
                Tok::Natural(2)
            ]
        )
    }

    #[test]
    fn test_tokenise_underscore_identifier() {
        assert_eq!(
            tok_unwrap("underscore_name"),
            vec![Tok::Identifier("underscore_name".into())],
        )
    }

    #[test]
    fn test_tokenise_string() {
        assert_eq!(
            tok_unwrap(r#"var = "string1" + "string2""#),
            vec![
                Tok::Identifier("var".into()),
                Tok::Operator(Operator::Assign),
                Tok::String("string1".into()),
                Tok::Operator(Operator::Add),
                Tok::String("string2".into())
            ]
        )
    }

    #[test]
    fn test_tokenise_decimal() {
        assert_eq!(tok_unwrap("3.14159"), vec![Tok::Decimal(3.14159)])
    }

    #[test]
    fn test_tokenise_decimal_call() {
        assert_eq!(
            tok_unwrap("floor(2.72)"),
            vec![
                Tok::identifier("floor"),
                Tok::ParenOpen,
                Tok::Decimal(2.72),
                Tok::ParenClose
            ]
        )
    }

    #[test]
    fn test_tokenise_all_ops() {
        for op in Operator::TOKENS {
            assert_eq!(read_token(op.chars()).unwrap().1, Tok::Operator(*op));
        }
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            tok_unwrap("2 + 3 # two plus three"),
            vec![
                Tok::Natural(2),
                Tok::Operator(Operator::Add),
                Tok::Natural(3)
            ]
        )
    }

    #[test]
    fn test_tokenise_multiline() {
        assert_eq!(
            tokenise(
                r#"
if 2 > 3 then
    print("wrong")
else
    print("right!")
                "#
                .trim()
            )
            .unwrap(),
            vec![
                Token::new(Tok::identifier("if"), 1, 1, 2),
                Token::new(Tok::Natural(2), 1, 4, 1),
                Token::new(Tok::Operator(Operator::GreaterThan), 1, 6, 1),
                Token::new(Tok::Natural(3), 1, 8, 1),
                Token::new(Tok::identifier("then"), 1, 10, 4),
                Token::new(Tok::identifier("print"), 2, 5, 5),
                Token::new(Tok::ParenOpen, 2, 10, 1),
                Token::new(Tok::String("wrong".into()), 2, 11, 7),
                Token::new(Tok::ParenClose, 2, 18, 1),
                Token::new(Tok::identifier("else"), 3, 1, 4),
                Token::new(Tok::identifier("print"), 4, 5, 5),
                Token::new(Tok::ParenOpen, 4, 10, 1),
                Token::new(Tok::String("right!".into()), 4, 11, 8),
                Token::new(Tok::ParenClose, 4, 19, 1)
            ]
        )
    }

    #[test]
    fn test_tokenise_escaped_string() {
        assert_eq!(
            tokenise("\"\\\"\"").unwrap(),
            vec![Token::new(Tok::String("\"".into()), 1, 1, 4)]
        )
    }

    #[test]
    fn test_tokenise_escaped_string_offsets() {
        assert_eq!(
            tokenise("\"\\\\\" \"\\\"\"").unwrap(),
            vec![
                Token::new(Tok::String("\\".into()), 1, 1, 4),
                Token::new(Tok::String("\"".into()), 1, 6, 4)
            ]
        )
    }
}
