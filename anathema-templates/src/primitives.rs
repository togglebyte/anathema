use std::fmt::Display;

use anathema_state::{CommonVal, Hex};

/// Primitive values such as booleans and integers.
/// These values are all static and resolved at eval time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    Bool(bool),
    Char(char),
    Int(i64),
    Float(f64),
    Hex(Hex),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(v) => write!(f, "{v}"),
            Self::Char(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Hex(Hex { r, g, b }) => write!(f, "r: {r}, g: {g}, b: {b}"),
        }
    }
}

macro_rules! from_value {
    ($from_type:tt, $variant:ident) => {
        impl From<$from_type> for Primitive {
            fn from(value: $from_type) -> Self {
                Self::$variant(value)
            }
        }
    };
}

from_value!(f64, Float);
from_value!(i64, Int);
from_value!(bool, Bool);
from_value!(char, Char);

impl From<(u8, u8, u8)> for Primitive {
    fn from(value: (u8, u8, u8)) -> Self {
        let (r, g, b) = value;
        Self::Hex(Hex { r, g, b })
    }
}

impl From<Primitive> for CommonVal<'_> {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Bool(val) => Self::Bool(val),
            Primitive::Char(val) => Self::Char(val),
            Primitive::Int(val) => Self::Int(val),
            Primitive::Float(val) => Self::Float(val),
            Primitive::Hex(hex) => Self::Hex(hex),
        }
    }
}
