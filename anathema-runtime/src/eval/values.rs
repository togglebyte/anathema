use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

use anathema_compiler::expressions::ExpressionId;
use anathema_compiler::{Color, Hex};
use anathema_store::remotecell::RemoteCell;

use crate::value::{AnonValue, ValueIndex};
use crate::Type;

/// A collection used by a for-loop
#[derive(Debug, Copy, Clone)]
pub struct Collection {
    pub(crate) expr: ValueIndex,
    len: u32,
    // inner: RemoteCell<TemplateValue<'bp>>,
    // pub key: SubKey,
}

impl Collection {
    pub(super) fn new(expr: ValueIndex, len: u32) -> Self {
        Self { expr, len }
    }

    pub(crate) fn len(&self) -> u32 {
        self.len
    }
}

/// This value can never be part of an evaluation chain, only the return value.
/// It should only ever be the final type that is held by a `Value`, at
/// the end of an evaluation
#[derive(Debug, PartialEq, Clone)]
pub enum TemplateValue<'bp> {
    /// Int
    Int(i64),
    /// Float
    Float(f64),
    /// Bool
    Bool(bool),
    /// Char
    Char(char),
    /// Hex
    Hex(Hex),
    /// Color
    Color(Color),
    /// Either a borrowed string from the template or an owned string
    /// from state
    Str(Cow<'bp, str>),
    /// Null...
    Null,

    // NOTE
    // The map is the final value, and is never used as part
    // of an index, for that reason the map doesn't hold any values.
    // TODO: is this true? what about variables binding to maps? e.g: let a = {a: 1}, let b = a.a
    /// A map of values
    Map(HashMap<&'bp str, TemplateValue<'bp>>),
    // Map,
    // NOTE
    // The attributes is the final value, and is never used as part
    // of an index, for that reason the attributes doesn't hold any values.
    /// Attributes
    Attributes,
    /// A collection of values
    List(Box<[TemplateValue<'bp>]>),
    /// A collection of values from state
    DynList(AnonValue),
    /// A map of values from state
    DynMap(AnonValue),
    /// A state
    Composite(AnonValue),
    /// A range
    Range {
        /// Start of the range
        start: i64,
        /// End of the range
        end: i64,
        /// Is the range inclusive or exclusive
        inclusive: bool,
    },
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

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub(crate) fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub(crate) fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub(crate) fn as_char(&self) -> Option<char> {
        match self {
            Self::Char(c) => Some(*c),
            _ => None,
        }
    }

    pub(crate) fn as_hex(&self) -> Option<Hex> {
        match self {
            Self::Hex(h) => Some(*h),
            _ => None,
        }
    }

    pub(crate) fn as_color(&self) -> Option<Color> {
        match self {
            Self::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn as_range(&self) -> Option<Range<i64>> {
        match self {
            Self::Range {
                start,
                end,
                inclusive: false,
            } => Some(*start..*end),
            Self::Range {
                start,
                end,
                inclusive: true,
            } => Some(*start..*end + 1),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
//   - From -
// -----------------------------------------------------------------------------
impl From<AnonValue> for TemplateValue<'static> {
    fn from(value: AnonValue) -> Self {
        let state = value.as_state();
        match value.type_info() {
            Type::Int => state.as_int().expect("type checked").into(),
            Type::Float => state.as_float().expect("type checked").into(),
            Type::Char => state.as_char().expect("type checked").into(),
            Type::String => state.as_str().expect("type checked").to_string().into(),
            Type::Bool => state.as_bool().expect("type checked").into(),
            Type::Hex => state.as_hex().expect("type checked").into(),
            Type::Color => state.as_color().expect("type checked").into(),
            Type::Map => TemplateValue::DynMap(value.clone()),
            Type::List => TemplateValue::DynList(value.clone()),
            Type::Composite => TemplateValue::Composite(value.clone()),
            Type::Unit => TemplateValue::Null,
            Type::Maybe => match state.as_maybe().expect("type checked").get() {
                Some(value) => value.into(),
                None => Self::Null,
            },
        }
    }
}

macro_rules! impl_from {
    ($t:ty, $variant:ident) => {
        impl From<$t> for TemplateValue<'static> {
            fn from(val: $t) -> Self {
                Self::$variant(val)
            }
        }
    };
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

impl From<Range<i64>> for TemplateValue<'static> {
    fn from(value: Range<i64>) -> Self {
        Self::Range {
            start: value.start,
            end: value.end,
            inclusive: false,
        }
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

#[cfg(test)]
pub mod test {
    use super::*;

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
    pub fn range(start: i64, end: i64, inclusive: bool) -> TemplateValue<'static> {
        TemplateValue::Range { start, end, inclusive }
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
}
