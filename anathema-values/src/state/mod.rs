pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State: std::fmt::Debug {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: &NodeId) -> ValueRef<'_>;

    #[doc(hidden)]
    fn get_value(&self, _: &NodeId) -> ValueRef<'_>
    where
        Self: Sized,
    {
        ValueRef::Map(self)
    }
}

/// This exists so you can have a view with a default state of a unit
impl State for () {
    fn get(&self, _: &Path, _: &NodeId) -> ValueRef<'_> {
        ValueRef::Empty
    }
}
