use std::borrow::Cow;
use std::ops::Deref;

use crate::{NodeId, Path, Collection};

pub trait State {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>>;

    fn get_no_sub(&self, key: &Path) -> Option<Cow<'_, str>>;

    fn get_collection(&self, key: &Path) -> Option<Collection>;
}

impl State for Box<dyn State> {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>> {
        self.deref().get(key, node_id)
    }

    fn get_no_sub(&self, key: &Path) -> Option<Cow<'_, str>> {
        self.deref().get_no_sub(key)
    }

    fn get_collection(&self, key: &Path) -> Option<Collection> {
        self.deref().get_collection(key)
    }
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>> {
        None
    }

    fn get_no_sub(&self, key: &Path) -> Option<Cow<'_, str>> {
        None
    }

    fn get_collection(&self, key: &Path) -> Option<Collection> {
        None
    }
}
