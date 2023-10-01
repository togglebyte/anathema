use std::fmt::{self, Display, Debug};
use std::rc::Rc;

use anathema_render::Color;

pub use self::num::Num;
pub use self::owned::Owned;
use crate::hashmap::HashMap;
use crate::map::Map;
use crate::{Collection, StateValue};

mod num;
mod owned;

// -----------------------------------------------------------------------------
//   - Value ref -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
pub enum ValueRef<'a> {
    Str(&'a str),
    Map(&'a dyn Collection),
    Owned(Owned),
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

impl<'a, T: Debug> From<&'a HashMap<String, StateValue<T>>> for ValueRef<'a>
where
    for<'b> ValueRef<'b>: From<&'b T>,
{
    fn from(value: &'a HashMap<String, StateValue<T>>) -> Self {
        Self::Map(value)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    fn from(value: &'a str) -> Self {
        ValueRef::Str(value)
    }
}

impl<'a, T: Into<Owned> + Copy> From<&'a T> for ValueRef<'a> {
    fn from(value: &'a T) -> Self {
        ValueRef::Owned((*value).into())
    }
}

// -----------------------------------------------------------------------------
//   - Value -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(Rc<str>),
    Owned(Owned),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{s}"),
            Self::Owned(owned) => write!(f, "{owned}"),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Owned(Owned::Bool(b))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Str(s.into())
    }
}

// -----------------------------------------------------------------------------
//   - TryFrom -
// -----------------------------------------------------------------------------
impl<'a> TryFrom<&'a Value> for &'a u64 {
    type Error = ();

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Owned(owned) => owned.try_into(),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<ValueRef<'a>> for &'a u64 {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Owned(owned) => panic!(), //owned.try_into(),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a str {
    type Error = ();

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Str(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<ValueRef<'a>> for &'a str {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            ValueRef::Str(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a Color {
    type Error = ();

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match value {
            Value::Owned(Owned::Color(col)) => Ok(col),
            _ => Err(()),
        }
    }
}

impl<'a> TryFrom<ValueRef<'a>> for &'a Color {
    type Error = ();

    fn try_from(value: ValueRef<'a>) -> Result<Self, Self::Error> {
        match value {
            // ValueRef::Str(s) => Ok(s),
            // _ => Err(())
            _ => panic!(),
        }
    }
}
