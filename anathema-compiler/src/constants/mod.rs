use anathema_values::{Path, ScopeValue};
pub(crate) use storage::Storage;

// use self::paths::Paths;
use self::strings::Strings;
// use self::texts::Texts;
use self::values::Values;

pub use self::strings::StringId;
pub use self::values::ValueId;
// pub use self::texts::TextId;

mod paths;
mod storage;
mod strings;
// mod texts;
mod values;

// -----------------------------------------------------------------------------
//   - Constants -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Constants {
    strings: Strings,
    values: Values,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            strings: Strings::empty(),
            values: Values::empty(),
        }
    }

    pub(crate) fn store_string(&mut self, string: impl Into<String>) -> StringId {
        self.strings.push(string.into())
    }

    pub fn store_value(&mut self, value: ScopeValue) -> ValueId {
        self.values.push(value)
    }

    pub fn lookup_string(&self, index: StringId) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    pub fn lookup_value(&self, index: ValueId) -> Option<&ScopeValue> {
        self.values.get(index)
    }
}
