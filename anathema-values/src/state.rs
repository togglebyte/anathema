use std::borrow::Cow;
use std::ops::Deref;

use crate::scope::StaticValue;
use crate::{Collection, NodeId, Path};

pub trait State {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, StaticValue>>;

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection>;
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, StaticValue>> {
        self.deref().get(key, node_id)
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection> {
        self.deref().get_collection(key, node_id)
    }
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, StaticValue>> {
        None
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Collection> {
        None
    }
}
