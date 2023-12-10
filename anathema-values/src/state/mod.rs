use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State : std::fmt::Debug {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>;

    #[doc(hidden)]
    // This is here to keep the proc macro happy
    fn get_value(&self, _: Option<&NodeId>) -> ValueRef<'_> {
        ValueRef::Empty
    }
}

/// This exists so you can have a view with a default state of a unit
impl State for () {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        ValueRef::Empty
    }
}
