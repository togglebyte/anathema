use std::fmt::Debug;

use anathema_render::Color;

pub use self::num::Num;
pub use self::owned::Owned;
use crate::hashmap::HashMap;
use crate::map::Map;
use crate::{Collection, List, Path, ValueExpr, State};

mod num;
mod owned;

// -----------------------------------------------------------------------------
//   - Value ref -
// -----------------------------------------------------------------------------
/// A value reference is either owned or referencing something
/// inside an expression.
#[derive(Debug, Clone, Default)]
pub enum ValueRef<'a> {
    Str(&'a str),
    Map(&'a dyn State),
    List(&'a dyn Collection),
    Expressions(&'a [ValueExpr]),
    ExpressionMap(&'a HashMap<String, ValueExpr>),
    Owned(Owned),
    /// A deferred lookup.
    /// This should only ever be a path into a state, and
    /// a state should never return a deferred value.
    Deferred(Path),
    #[default]
    Empty,
}

impl<'a> ValueRef<'a> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Str(s) => !s.is_empty(),
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

impl<'a> Into<ValueRef<'a>> for &'a String {
    fn into(self) -> ValueRef<'a> {
        ValueRef::Str(self)
    }
}

impl<'a> Into<ValueRef<'a>> for &'a str {
    fn into(self) -> ValueRef<'a> {
        ValueRef::Str(self)
    }
}

impl<'a> Into<ValueRef<'a>> for Owned {
    fn into(self) -> ValueRef<'a> {
        ValueRef::Owned(self)
    }
}

impl<'a, T> From<&'a T> for ValueRef<'a>
where
    &'a T: Into<Owned>,
{
    fn from(val: &'a T) -> ValueRef<'a> {
        Self::Owned(val.into())
    }
}

impl<'a, T> Into<ValueRef<'a>> for &'a List<T>
where
    T: Debug,
    for<'b> &'b T: Into<ValueRef<'b>>,
{
    fn into(self) -> ValueRef<'a> {
        ValueRef::List(self)
    }
}

impl<'a, T> Into<ValueRef<'a>> for &'a Map<T>
where
    T: Debug,
    for<'b> &'b T: Into<ValueRef<'b>>,
{
    fn into(self) -> ValueRef<'a> {
        ValueRef::Map(self)
    }
}

impl<'a> Into<ValueRef<'a>> for &'a dyn State {
    fn into(self) -> ValueRef<'a> {
        ValueRef::Map(self)
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
                    ValueRef::Owned(Owned::Num(Num::Signed(num))) => Ok(num as $t),
                    ValueRef::Owned(Owned::Num(Num::Unsigned(num))) => Ok(num as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! float_try_from {
    ($t:ty) => {
        impl TryFrom<ValueRef<'_>> for $t {
            type Error = ();

            fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
                match value {
                    ValueRef::Owned(Owned::Num(Num::Float(num))) => Ok(num as $t),
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

float_try_from!(f64);
float_try_from!(f32);

impl TryFrom<ValueRef<'_>> for String {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s.to_string()),
            _ => Err(()),
        }
    }
}

impl<'expr> TryFrom<ValueRef<'expr>> for &'expr str {
    type Error = ();

    fn try_from(value: ValueRef<'expr>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s),
            _ => Err(()),
        }
    }
}
