use std::borrow::Cow;
use std::ops::Deref;

pub use self::list::List;
pub use self::map::Map;
pub use self::scope::{Collection, Context, Scope, ScopeValue};
pub use self::slab::Slab;
pub use self::value::Value;
use crate::Path;

mod list;
mod map;
mod scope;
mod slab;
mod value;

pub trait State {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>>;

    fn get_collection(&self, key: &Path) -> Option<Collection>;
}

/// Implementation of `State` for a unit.
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>> {
        None
    }

    fn get_collection(&self, key: &Path) -> Option<Collection> {
        None
    }
}
