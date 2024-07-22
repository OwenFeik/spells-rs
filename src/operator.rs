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
    And,
    Or,
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
        Operator::And,
        Operator::Or,
    ];

    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Sentinel => 0,
            Operator::Discard => 1,
            Operator::Assign => 2,
            Operator::And => 3,
            Operator::Or => 3,
            Operator::GreaterThan => 4,
            Operator::LessThan => 4,
            Operator::GreaterEqual => 4,
            Operator::LessEqual => 4,
            Operator::Equal => 4,
            Operator::Add => 5,
            Operator::Sub => 5,
            Operator::Mul => 6,
            Operator::Div => 6,
            Operator::Neg => 7,
            Operator::Adv => 7,
            Operator::DisAdv => 7,
            Operator::Sort => 7,
            Operator::Exp => 8,
            Operator::Keep => 9,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => false,
            Operator::Discard => true,
            Operator::And => true,
            Operator::Or => true,
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
            Operator::And => true,
            Operator::Or => true,
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
