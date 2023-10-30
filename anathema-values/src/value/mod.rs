use std::fmt::Debug;

use anathema_render::Color;

pub use self::num::Num;
pub use self::owned::Owned;
use crate::hashmap::HashMap;
use crate::map::Map;
use crate::{Collection, List, Path, ValueExpr};

mod num;
mod owned;

// -----------------------------------------------------------------------------
//   - Value ref -
// -----------------------------------------------------------------------------
/// A value reference is either owned or referencing something
/// inside an expression.
#[derive(Debug, Clone)]
pub enum ValueRef<'a> {
    Str(&'a str),
    Map(&'a dyn Collection),
    List(&'a dyn Collection),
    Expressions(&'a [ValueExpr]),
    ExpressionMap(&'a HashMap<String, ValueExpr>),
    Owned(Owned),
    /// A deferred lookup. This should only ever
    /// be a path into a state
    Deferred(Path),
}

impl<'a> ValueRef<'a> {
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

impl<'a> PartialEq for ValueRef<'a> {
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
impl<'a, T: Debug> From<&'a Map<T>> for ValueRef<'a>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'a Map<T>) -> Self {
        Self::Map(value)
    }
}

impl<'a, T: Debug> From<&'a List<T>> for ValueRef<'a>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'a List<T>) -> Self {
        Self::List(value)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    fn from(value: &'a str) -> Self {
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

macro_rules! num_try_from {
    ($t:ty, $idn:ident) => {
        impl TryFrom<ValueRef<'_>> for $t {
            type Error = ();

            fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
                match value {
                    ValueRef::Owned(Owned::Num(Num::$idn(num))) => Ok(num as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! val_try_from {
    ($t:ty, $idn:ident) => {
        impl TryFrom<ValueRef<'_>> for $t {
            type Error = ();

            fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
                match value {
                    ValueRef::Owned(Owned::$idn(val)) => Ok(val),
                    _ => Err(()),
                }
            }
        }
    };
}

val_try_from!(bool, Bool);
val_try_from!(Color, Color);

num_try_from!(usize, Unsigned);
num_try_from!(u64, Unsigned);
num_try_from!(u32, Unsigned);
num_try_from!(u16, Unsigned);
num_try_from!(u8, Unsigned);

num_try_from!(isize, Signed);
num_try_from!(i64, Signed);
num_try_from!(i32, Signed);
num_try_from!(i16, Signed);
num_try_from!(i8, Signed);

num_try_from!(f64, Float);
num_try_from!(f32, Float);


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
