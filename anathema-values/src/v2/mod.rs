use std::borrow::Cow;

pub use self::list::List;
pub use self::map::Map;
pub use self::scope::{Collection, Context, Scope, ScopeValue};
pub use self::value::Value;
pub use self::slab::Slab;
use crate::Path;

mod list;
mod map;
mod scope;
mod slab;
mod value;


pub trait State {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>>;

    fn get_typed<T>(&self, key: &Path) -> Option<T>;

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

    fn get_typed<T>(&self, key: &Path) -> Option<T> {
        None
    }
}
