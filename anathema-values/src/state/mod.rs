use std::borrow::Cow;
use std::ops::Deref;

pub use self::value::{Change, StateValue};
use crate::{Collection, NodeId, Path, ValueRef};

mod value;

pub trait State {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>;

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection>;
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        self.deref().get(key, node_id)
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection> {
        self.deref().get_collection(key, node_id)
    }
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        None
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection> {
        None
    }
}
