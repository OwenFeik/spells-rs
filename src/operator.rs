#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Sentinel,
    Assign,
    Discard,
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Neg,
    Keep,
    Adv,
    DisAdv,
    Sort,
    Equal,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
}

impl Operator {
    // Operators which are produced context-free by the tokeniser.
    pub const TOKENS: &'static [Operator] = &[
        Operator::Assign,
        Operator::Discard,
        Operator::Add,
        Operator::Sub,
        Operator::Mul,
        Operator::Div,
        Operator::Exp,
        Operator::Equal,
        Operator::GreaterThan,
        Operator::LessThan,
        Operator::GreaterEqual,
        Operator::LessEqual,
    ];

    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Sentinel => 0,
            Operator::Assign => 1,
            Operator::Discard => 2,
            Operator::GreaterThan => 3,
            Operator::LessThan => 3,
            Operator::GreaterEqual => 3,
            Operator::LessEqual => 3,
            Operator::Equal => 3,
            Operator::Add => 4,
            Operator::Sub => 4,
            Operator::Mul => 5,
            Operator::Div => 5,
            Operator::Neg => 6,
            Operator::Adv => 6,
            Operator::DisAdv => 6,
            Operator::Sort => 6,
            Operator::Exp => 7,
            Operator::Keep => 8,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => false,
            Operator::Discard => true,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => false,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
            Operator::Sort => false,
            Operator::Equal => true,
            Operator::GreaterThan => true,
            Operator::LessThan => true,
            Operator::GreaterEqual => true,
            Operator::LessEqual => true,
        }
    }

    pub fn is_binary(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => true,
            Operator::Discard => true,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => true,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
            Operator::Sort => false,
            Operator::Equal => true,
            Operator::GreaterThan => true,
            Operator::LessThan => true,
            Operator::GreaterEqual => true,
            Operator::LessEqual => true,
        }
    }

    pub fn is_unary(&self) -> bool {
        matches!(
            self,
            Operator::Neg | Operator::Adv | Operator::DisAdv | Operator::Sort
        )
    }

    pub fn is_unary_postfix(&self) -> bool {
        matches!(self, Operator::Adv | Operator::DisAdv | Operator::Sort)
    }

    pub fn greater(left: &Self, right: &Self) -> bool {
        if left.is_binary() && right.is_binary() {
            (left.precedence() > right.precedence())
                || (left.left_associative() && left.precedence() == right.precedence())
        } else if left.is_unary() && right.is_binary() || right.is_unary_postfix() {
            left.precedence() >= right.precedence()
        } else {
            false
        }
    }

    pub fn str(&self) -> &str {
        match self {
            Operator::Sentinel => "@",
            Operator::Assign => "=",
            Operator::Discard => ";",
            Operator::Add => "+",
            Operator::Sub => "-",
            Operator::Mul => "*",
            Operator::Div => "/",
            Operator::Exp => "^",
            Operator::Neg => "-",
            Operator::Keep => "k",
            Operator::Adv => "a",
            Operator::DisAdv => "d",
            Operator::Sort => "s",
            Operator::Equal => "==",
            Operator::GreaterThan => ">",
            Operator::LessThan => "<",
            Operator::GreaterEqual => ">=",
            Operator::LessEqual => "<=",
        }
    }
}
