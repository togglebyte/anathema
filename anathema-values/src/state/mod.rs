use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>;

    #[doc(hidden)]
    fn __anathema_get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>  {
        self.get(key, node_id)
    }
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        self.deref().get(key, node_id)
    }
}

pub trait BlanketGet {
    fn __anathema_get_value(&self, node_id: Option<&NodeId>) -> ValueRef<'static> {
        ValueRef::Empty
    }

    fn __anathema_get<'a>(&self, key: &'a Path, node_id: Option<&NodeId>) -> ValueRef<'static> {
        ValueRef::Empty
    }

    fn __anathema_subscribe(&self, node_id: NodeId) {}
}

impl<T> BlanketGet for &T {}

