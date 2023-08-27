use std::borrow::Cow;

pub use self::list::List;
pub use self::map::Map;
pub use self::value::Value;
use crate::Path;

mod list;
mod map;
mod value;

pub trait State {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>>;
}

/// Implementation of `State` for a unit. 
/// This will always return `None` and should only be used for testing purposes
impl State for () {
    fn get(&self, key: &Path) -> Option<Cow<'_, str>> {
        None
    }
}
