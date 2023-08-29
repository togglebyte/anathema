use std::borrow::Cow;

pub use self::list::List;
pub use self::map::Map;
pub use self::value::Value;
pub use self::scope::{Scope, ScopeValue, Context};
use crate::Path;

mod list;
mod map;
mod value;
mod scope;

pub trait State {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>>;

    fn get_typed<T>(&self, key: &Path) -> Option<T>;
}

/// Implementation of `State` for a unit. 
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>> {
        None
    }

    fn get_typed<T>(&self, key: &Path) -> Option<T> {
        None
    }
}
