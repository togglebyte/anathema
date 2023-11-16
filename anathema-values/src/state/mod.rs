use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{NodeId, Path, ValueRef};

mod value;

pub trait State {
    /// Get a value reference from the state
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_>;
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        self.deref().get(key, node_id)
    }
}
