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
    // Map(HashMap<&'bp str, TemplateValue<'bp>>),
    Map,
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
