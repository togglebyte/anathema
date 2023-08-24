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
