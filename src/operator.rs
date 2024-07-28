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
    Equal,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Not,
}

impl Operator {
    // Operators which are produced context-free by the tokeniser.
    // NB it is important that these are ordered longest-to-shortest.
    pub const TOKENS: &'static [Operator] = &[
        Operator::Equal,        // ==
        Operator::GreaterEqual, // >=
        Operator::LessEqual,    // <=
        Operator::GreaterThan,  // >
        Operator::LessThan,     // <
        Operator::Assign,       // =
        Operator::Discard,      // ;
        Operator::Add,          // +
        Operator::Sub,          // -
        Operator::Mul,          // *
        Operator::Div,          // /
        Operator::Exp,          // ^
        Operator::And,          // &
        Operator::Or,           // |
        Operator::Not,          // !
    ];

    pub const ROLL_SUFFIX_TOKENS: &'static [Operator] = &[Self::Keep, Self::Adv, Self::DisAdv];

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
            Operator::Not => 7,
            Operator::Neg => 7,
            Operator::Adv => 7,
            Operator::DisAdv => 7,
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
            Operator::Not => false,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => false,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
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
            Operator::Not => false,
            Operator::Add => true,
            Operator::Sub => true,
            Operator::Mul => true,
            Operator::Div => true,
            Operator::Exp => true,
            Operator::Neg => false,
            Operator::Keep => true,
            Operator::Adv => false,
            Operator::DisAdv => false,
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
            Operator::Not | Operator::Neg | Operator::Adv | Operator::DisAdv
        )
    }

    pub fn is_unary_prefix(&self) -> bool {
        matches!(self, Operator::Not | Operator::Neg)
    }

    pub fn is_unary_postfix(&self) -> bool {
        matches!(self, Operator::Adv | Operator::DisAdv)
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

    pub fn chars(&self) -> &[char] {
        match self {
            Operator::Sentinel => &['@'],
            Operator::Assign => &['='],
            Operator::Discard => &[';'],
            Operator::And => &['&'],
            Operator::Or => &['|'],
            Operator::Not => &['!'],
            Operator::Add => &['+'],
            Operator::Sub => &['-'],
            Operator::Mul => &['*'],
            Operator::Div => &['/'],
            Operator::Exp => &['^'],
            Operator::Neg => &['-'],
            Operator::Keep => &['k'],
            Operator::Adv => &['a'],
            Operator::DisAdv => &['d'],
            Operator::Equal => &['=', '='],
            Operator::GreaterThan => &['>'],
            Operator::LessThan => &['<'],
            Operator::GreaterEqual => &['>', '='],
            Operator::LessEqual => &['<', '='],
        }
    }

    pub fn str(&self) -> String {
        let mut s = String::new();
        for &c in self.chars() {
            s.push(c);
        }
        s
    }
}
