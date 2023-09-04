use std::cell::RefCell;

pub use self::list::List;
pub use self::map::Map;
pub use self::scope::{Collection, Context, Scope, ScopeValue};
pub use self::slab::Slab;
pub use self::value::Value;
pub use self::id::NodeId;
pub use self::state::State;
use crate::Path;

mod id;
mod list;
mod map;
mod scope;
mod slab;
mod value;
mod state;

thread_local! {
    pub static DIRTY_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<NodeId> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}

