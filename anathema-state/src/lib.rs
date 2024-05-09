pub use anathema_state_derive::State;
use anathema_store::slab::Key;

pub use crate::common::{CommonString, CommonVal};
pub use crate::numbers::Number;
pub use crate::states::{State, StateId, Stateless, States};
pub use crate::store::{
    debug, drain_changes, drain_futures, register_future, Change, Changes, FutureValues, Subscriber,
};
pub use crate::value::{List, Map, PendingValue, SharedState, Value, ValueRef};

mod common;
mod numbers;
mod states;
mod store;
mod value;

// -----------------------------------------------------------------------------
//   - Macro requirements -
// -----------------------------------------------------------------------------
#[allow(unused_extern_crates)]
extern crate self as anathema;
#[allow(unused_imports)]
pub use crate as state;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Path<'e> {
    Key(&'e str),
    Index(usize),
}

impl From<usize> for Path<'_> {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl<'a> From<&'a str> for Path<'a> {
    fn from(value: &'a str) -> Self {
        Self::Key(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hex {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8, u8)> for Hex {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}
