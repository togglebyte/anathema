use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::Deref;

pub use self::list::List;
pub use self::map::Map;
pub use self::scope::{Collection, Context, Scope, ScopeValue};
pub use self::slab::Slab;
pub use self::value::Value;
pub use self::id::NodeId;
use crate::Path;

mod id;
mod list;
mod map;
mod scope;
mod slab;
mod value;

thread_local! {
    pub static DIRTY_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<NodeId> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}


pub trait State {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>>;

    fn get_collection(&self, key: &Path) -> Option<Collection>;
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>> {
        None
    }

    fn get_collection(&self, key: &Path) -> Option<Collection> {
        None
    }
}
