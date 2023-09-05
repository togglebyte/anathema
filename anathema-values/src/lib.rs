use std::cell::RefCell;

pub use self::id::NodeId;
pub use self::list::List;
pub use self::map::Map;
pub use self::path::Path;
pub use self::scope::{Collection, Context, Scope, ScopeValue};
pub use self::slab::Slab;
pub use self::state::State;
pub use self::value::Value;

pub mod hashmap;
mod path;

mod id;
mod list;
mod map;
mod scope;
mod slab;
mod state;
mod value;

thread_local! {
    pub static DIRTY_NODES: RefCell<Vec<NodeId>> = Default::default();
}

pub fn drain_dirty_nodes() -> Vec<NodeId> {
    DIRTY_NODES.with(|nodes| nodes.borrow_mut().drain(..).collect())
}

// #[cfg(testing)]
pub mod testing;
