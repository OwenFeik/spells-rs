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
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::Sentinel => 0,
            Operator::Assign => 1,
            Operator::Add => 2,
            Operator::Sub => 2,
            Operator::Mul => 3,
            Operator::Div => 3,
            Operator::Exp => 5,
            Operator::Neg => 4,
            Operator::Keep => 6,
            Operator::Adv => 4,
            Operator::DisAdv => 4,
            Operator::Sort => 4,
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

    pub fn char(&self) -> char {
        match self {
            Operator::Sentinel => '@',
            Operator::Assign => '=',
            Operator::Add => '+',
            Operator::Sub => '-',
            Operator::Mul => '*',
            Operator::Div => '/',
            Operator::Exp => '^',
            Operator::Neg => '-',
            Operator::Keep => 'k',
            Operator::Adv => 'a',
            Operator::DisAdv => 'd',
            Operator::Sort => 's',
        }
    }
}
