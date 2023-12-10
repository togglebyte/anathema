use std::fmt::{self, Display};

use anathema_render::Color;

use crate::Num;

#[derive(Debug, Clone, Copy, PartialEq)]
// TODO: rename to Primitive
pub enum Owned {
    Num(Num),
    Bool(bool),
    Color(Color),
}

impl<T: Into<Num>> From<T> for Owned {
    fn from(val: T) -> Self {
        Self::Num(val.into())
    }
}

impl From<bool> for Owned {
    fn from(val: bool) -> Self {
        Self::Bool(val)
    }
}

impl From<&bool> for Owned {
    fn from(val: &bool) -> Self {
        Self::Bool(*val)
    }
}

impl From<Color> for Owned {
    fn from(val: Color) -> Self {
        Self::Color(val)
    }
}

impl From<&Color> for Owned {
    fn from(val: &Color) -> Self {
        Self::Color(*val)
    }
}

impl TryFrom<Owned> for Color {
    type Error = ();

    fn try_from(value: Owned) -> Result<Self, Self::Error> {
        match value {
            Owned::Color(color) => Ok(color),
            _ => Err(()),
        }
    }
}

impl TryFrom<Owned> for usize {
    type Error = ();

    fn try_from(value: Owned) -> Result<Self, Self::Error> {
        match value {
            Owned::Num(Num::Unsigned(num)) => Ok(num as usize),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a Owned> for &'a u64 {
    type Error = ();

    fn try_from(value: &'a Owned) -> Result<Self, Self::Error> {
        match value {
            Owned::Num(Num::Unsigned(num)) => Ok(num),
            _ => Err(()),
        }
    }
}

// TODO: add the rest of the types to TryFrom

impl Display for Owned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(num) => write!(f, "{num}"),
            Self::Color(color) => write!(f, "{color:?}"),
            Self::Bool(b) => write!(f, "{b:?}"),
        }
    }
}
