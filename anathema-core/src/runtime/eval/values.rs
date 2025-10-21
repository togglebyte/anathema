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

impl_from!(i64, Int);
impl_from!(f64, Float);
impl_from!(bool, Bool);
impl_from!(char, Char);
impl_from!(Hex, Hex);
impl_from!(Color, Color);

impl From<std::ops::Range<usize>> for TemplateValue<'static> {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self::Range(value.start, value.end)
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
