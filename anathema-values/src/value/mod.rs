use std::fmt::{Debug};



use anathema_render::Color;

pub use self::num::Num;
pub use self::owned::Owned;
use crate::hashmap::HashMap;
use crate::map::Map;
use crate::{Collection, List, ValueExpr};

mod num;
mod owned;

// -----------------------------------------------------------------------------
//   - Value ref -
// -----------------------------------------------------------------------------
/// A value reference is either owned or referencing something
/// inside an expression.
#[derive(Debug, Copy, Clone)]
pub enum ValueRef<'expr> {
    Str(&'expr str),
    Map(&'expr dyn Collection),
    List(&'expr dyn Collection),
    Expressions(&'expr [ValueExpr]),
    ExpressionMap(&'expr HashMap<String, ValueExpr>),
    Owned(Owned),
}

impl<'expr> ValueRef<'expr> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Str(s) => s.is_empty(),
            Self::Owned(Owned::Bool(b)) => *b,
            Self::Owned(Owned::Num(Num::Unsigned(n))) => *n > 0,
            Self::Owned(Owned::Num(Num::Signed(n))) => *n > 0,
            _ => false,
        }
    }
}

impl<'expr> PartialEq for ValueRef<'expr> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Str(lhs), Self::Str(rhs)) => lhs == rhs,
            (Self::Owned(lhs), Self::Owned(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

// -----------------------------------------------------------------------------
//   - From for value ref -
// -----------------------------------------------------------------------------
impl<'expr, T: Debug> From<&'expr Map<T>> for ValueRef<'expr>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'expr Map<T>) -> Self {
        Self::Map(value)
    }
}

impl<'expr, T: Debug> From<&'expr List<T>> for ValueRef<'expr>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'expr List<T>) -> Self {
        Self::List(value)
    }
}

impl<'expr> From<&'expr str> for ValueRef<'expr> {
    fn from(value: &'expr str) -> Self {
        ValueRef::Str(value)
    }
}

impl<T: Into<Owned> + Copy> From<&T> for ValueRef<'_> {
    fn from(value: &T) -> Self {
        ValueRef::Owned((*value).into())
    }
}

// -----------------------------------------------------------------------------
//   - TryFrom -
// -----------------------------------------------------------------------------
impl TryFrom<ValueRef<'_>> for u64 {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(Owned::Num(Num::Unsigned(num))) => Ok(num),
            _ => Err(()),
        }
    }
}

impl TryFrom<ValueRef<'_>> for bool {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(Owned::Bool(b)) => Ok(b),
            _ => Err(()),
        }
    }
}

impl TryFrom<ValueRef<'_>> for usize {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(Owned::Num(Num::Unsigned(num))) => Ok(num as usize),
            _ => Err(()),
        }
    }
}

impl TryFrom<ValueRef<'_>> for String {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s.to_string()),
            _ => Err(()),
        }
    }
}

impl<'epr> TryFrom<ValueRef<'epr>> for &'epr str {
    type Error = ();

    fn try_from(value: ValueRef<'epr>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<ValueRef<'_>> for Color {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(Owned::Color(color)) => Ok(color),
            _ => Err(())
        }
    }
}
