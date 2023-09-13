use super::*;

#[test]
fn test_tokenise_roll() {
    assert_eq!(tokenise("1d4"), vec![Token::Roll(1, 4)]);
    assert_eq!(tokenise("d4"), vec![Token::Roll(1, 4)]);
    assert_eq!(tokenise("8d8"), vec![Token::Roll(8, 8)]);
    assert_eq!(tokenise("d20"), vec![Token::Roll(1, 20)]);
    assert_eq!(tokenise("d20 d20"), vec![Token::Roll(1, 20), Token::Roll(1, 20)]);
}

#[test]
fn test_tokenise_ops() {
    assert_eq!(
        tokenise("+ - * / ^"),
        vec![Token::Plus, Token::Minus, Token::Times, Token::Divide, Token::Exp]
    );
    assert_eq!(
        tokenise("a d k s"),
        vec![Token::Advantage, Token::Disadvantage, Token::Keep, Token::Sort]
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
