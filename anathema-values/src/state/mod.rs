// -----------------------------------------------------------------------------
//   - Notes about state -
//   State can either belong to a `View` or be passed to a `View`.
//   Any state owned by a view has to be returned by `View::state()` to be
//   accessible inside the templates.
//
//   State owned by the `View` is referred to as Internal State.
//   State passed to the `View` is External State.
// -----------------------------------------------------------------------------
pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State: std::fmt::Debug {
    /// Get a value reference from the state
    fn state_get(&self, key: &Path, node_id: &NodeId) -> ValueRef<'_>;

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
    fn state_get(&self, _: &Path, _: &NodeId) -> ValueRef<'_> {
        ValueRef::Empty
    }
}
