use std::fmt::{self, Display, Formatter};
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operator {
    LParen,
    RParen,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,
    ModEqual,
    Equal,
    EqualEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Not,
    And,
    Or,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::PlusEqual => write!(f, "+="),
            Self::MinusEqual => write!(f, "-="),
            Self::MulEqual => write!(f, "*="),
            Self::DivEqual => write!(f, "/="),
            Self::ModEqual => write!(f, "%="),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "=="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::Not => write!(f, "!"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}
