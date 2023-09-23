use std::fmt::{self, Display};

use anathema_render::Color;

use crate::Num;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Owned {
    Num(Num),
    Bool(bool),
    Color(Color),
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
            _ => Err(())
        }
    }
}

impl Display for Owned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Num(num) => write!(f, "{num}"),
            Self::Color(color) => write!(f, "{color:?}"),
            Self::Bool(b) => write!(f, "{b:?}"),
        }
    }
}
