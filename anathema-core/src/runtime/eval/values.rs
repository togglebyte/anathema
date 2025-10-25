use std::borrow::Cow;
use std::collections::HashMap;

use anathema_state::{AnonValue, Color, Hex};

/// This value can never be part of an evaluation chain, only the return value.
/// It should only ever be the final type that is held by a `Value`, at
/// the end of an evaluation
#[derive(Debug, PartialEq, Clone)]
pub enum TemplateValue<'bp> {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Hex(Hex),
    Color(Color),
    Str(Cow<'bp, str>),
    Null,

    // NOTE
    // The map is the final value, and is never used as part
    // of an index, for that reason the map doesn't hold any values.
    // TODO: is this true? what about variables binding to maps? e.g: let a = {a: 1}, let b = a.a
    Map(HashMap<&'bp str, TemplateValue<'bp>>),
    // Map,
    // NOTE
    // The attributes is the final value, and is never used as part
    // of an index, for that reason the attributes doesn't hold any values.
    Attributes,
    List(Box<[TemplateValue<'bp>]>),
    DynList(AnonValue),
    DynMap(AnonValue),
    Composite(AnonValue),
    Range(usize, usize),
}

impl TemplateValue<'_> {
    pub(crate) fn truthiness(&self) -> bool {
        match self {
            Self::Int(0) | Self::Float(0.0) | Self::Bool(false) => false,
            Self::Str(cow) if cow.is_empty() => false,
            Self::Null => false,
            Self::List(list) if list.is_empty() => false,
            Self::DynList(list) => {
                let state = list.as_state();
                let Some(state) = state.as_any_list() else { return false };
                !state.is_empty()
            }
            Self::DynMap(map) => {
                let state = map.as_state();
                let Some(state) = state.as_any_map() else { return false };
                !state.is_empty()
            }
            _ => true,
        }
    }
}

// -----------------------------------------------------------------------------
//   - From -
// -----------------------------------------------------------------------------
macro_rules! impl_from {
    ($t:ty, $variant:ident) => {
        impl From<$t> for TemplateValue<'static> {
            fn from(val: $t) -> Self {
                Self::$variant(val)
            }
        }
    }
}

impl_from!(bool, Bool);
impl_from!(char, Char);
impl_from!(Hex, Hex);
impl_from!(Color, Color);

macro_rules! from_int {
    ($int:ty) => {
        impl From<$int> for TemplateValue<'_> {
            fn from(value: $int) -> Self {
                TemplateValue::Int(value as i64)
            }
        }
    };
}

from_int!(i64);
from_int!(i32);
from_int!(i16);
from_int!(i8);
from_int!(u64);
from_int!(u32);
from_int!(u16);
from_int!(u8);

impl From<f64> for TemplateValue<'_> {
    fn from(value: f64) -> Self {
        TemplateValue::Float(value)
    }
}

impl From<f32> for TemplateValue<'_> {
    fn from(value: f32) -> Self {
        TemplateValue::Float(value as f64)
    }
}

impl From<std::ops::Range<usize>> for TemplateValue<'static> {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self::Range(value.start, value.end)
    }
}

impl<'bp, T> From<Vec<T>> for TemplateValue<'bp>
where
    T: Into<TemplateValue<'bp>>,
{
    fn from(value: Vec<T>) -> Self {
        let list = value.into_iter().map(T::into).collect();
        TemplateValue::List(list)
    }
}

impl<'a> From<&'a str> for TemplateValue<'a> {
    fn from(value: &'a str) -> Self {
        TemplateValue::Str(Cow::Borrowed(value))
    }
}

impl From<String> for TemplateValue<'_> {
    fn from(value: String) -> Self {
        TemplateValue::Str(value.into())
    }
}

// -----------------------------------------------------------------------------
//   - Primitives -
// -----------------------------------------------------------------------------
pub fn num(value: i64) -> TemplateValue<'static> {
    TemplateValue::Int(value)
}

pub fn float(value: f64) -> TemplateValue<'static> {
    TemplateValue::Float(value)
}

pub fn chr(value: char) -> TemplateValue<'static> {
    TemplateValue::Char(value)
}

pub fn hex(value: Hex) -> TemplateValue<'static> {
    TemplateValue::Hex(value)
}

pub fn color(value: Color) -> TemplateValue<'static> {
    TemplateValue::Color(value)
}

pub fn boolean(value: bool) -> TemplateValue<'static> {
    TemplateValue::Bool(value)
}

pub fn strlit<'a>(value: &'a str) -> TemplateValue<'a> {
    TemplateValue::Str(value.into())
}

// -----------------------------------------------------------------------------
//   - List, map and range -
// -----------------------------------------------------------------------------
pub fn range(lhs: usize, rhs: usize) -> TemplateValue<'static> {
    TemplateValue::Range(lhs, rhs)
}

pub fn list<'a, T: Into<TemplateValue<'a>>>(list: impl IntoIterator<Item = T>) -> TemplateValue<'a> {
    let list = list.into_iter().map(Into::into).collect();
    TemplateValue::List(list)
}

pub fn map<'a, T: Into<TemplateValue<'a>>>(map: HashMap<&'static str, T>) -> TemplateValue<'a> {
    let mut hm = HashMap::with_capacity(map.len());
    for (k, v) in map.into_iter() {
        hm.insert(k, v.into());
    }
    TemplateValue::Map(hm)
}

// -----------------------------------------------------------------------------
//   - Try From -
// -----------------------------------------------------------------------------
macro_rules! try_from_valuekind {
    ($t:ty, $kind:ident) => {
        impl TryFrom<&TemplateValue<'_>> for $t {
            type Error = ();

            fn try_from(value: &TemplateValue<'_>) -> Result<Self, Self::Error> {
                match value {
                    TemplateValue::$kind(val) => Ok(*val),
                    _ => Err(()),
                }
            }
        }
    };
}

macro_rules! try_from_valuekind_int {
    ($t:ty, $kind:ident) => {
        impl TryFrom<&TemplateValue<'_>> for $t {
            type Error = ();

            fn try_from(value: &TemplateValue<'_>) -> Result<Self, Self::Error> {
                match value {
                    TemplateValue::$kind(val) => Ok(*val as $t),
                    _ => Err(()),
                }
            }
        }
    };
}

try_from_valuekind!(i64, Int);
try_from_valuekind!(f64, Float);
try_from_valuekind!(bool, Bool);
try_from_valuekind!(char, Char);
try_from_valuekind!(Hex, Hex);
try_from_valuekind!(Color, Color);

try_from_valuekind_int!(usize, Int);
try_from_valuekind_int!(isize, Int);
try_from_valuekind_int!(i32, Int);
try_from_valuekind_int!(f32, Float);
try_from_valuekind_int!(i16, Int);
try_from_valuekind_int!(i8, Int);
try_from_valuekind_int!(u32, Int);
try_from_valuekind_int!(u64, Int);
try_from_valuekind_int!(u16, Int);
try_from_valuekind_int!(u8, Int);

impl<'a, 'bp> TryFrom<&'a TemplateValue<'bp>> for &'a str {
    type Error = ();

    fn try_from(value: &'a TemplateValue<'bp>) -> Result<Self, Self::Error> {
        match value {
            TemplateValue::Str(Cow::Borrowed(val)) => Ok(val),
            TemplateValue::Str(Cow::Owned(val)) => Ok(val.as_str()),
            _ => Err(()),
        }
    }
}
