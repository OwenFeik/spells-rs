#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Sentinel,
    Assign,
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
            Operator::GreaterThan => 2,
            Operator::LessThan => 2,
            Operator::GreaterEqual => 2,
            Operator::LessEqual => 2,
            Operator::Equal => 2,
            Operator::Add => 3,
            Operator::Sub => 3,
            Operator::Mul => 4,
            Operator::Div => 4,
            Operator::Neg => 5,
            Operator::Adv => 5,
            Operator::DisAdv => 5,
            Operator::Sort => 5,
            Operator::Exp => 6,
            Operator::Keep => 7,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => false,
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
