#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Sentinel,
    Assign,
    Define,
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
        Operator::Define,       // :=
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
            Operator::Define => 1,
            Operator::Discard => 2,
            Operator::Assign => 3,
            Operator::And => 4,
            Operator::Or => 4,
            Operator::GreaterThan => 5,
            Operator::LessThan => 5,
            Operator::GreaterEqual => 5,
            Operator::LessEqual => 5,
            Operator::Equal => 5,
            Operator::Add => 6,
            Operator::Sub => 6,
            Operator::Mul => 7,
            Operator::Div => 7,
            Operator::Not => 8,
            Operator::Neg => 8,
            Operator::Adv => 8,
            Operator::DisAdv => 8,
            Operator::Exp => 9,
            Operator::Keep => 10,
        }
    }

    fn left_associative(&self) -> bool {
        match self {
            Operator::Sentinel => false,
            Operator::Assign => false,
            Operator::Define => false,
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
            Operator::Define => true,
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

    pub fn chars(&self) -> &'static [char] {
        match self {
            Operator::Sentinel => &['@'],
            Operator::Assign => &['='],
            Operator::Define => &[':', '='],
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
