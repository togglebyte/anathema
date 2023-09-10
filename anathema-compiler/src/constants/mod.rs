use anathema_values::{Path, ScopeValue};
pub(crate) use storage::Storage;

// use self::paths::Paths;
use self::strings::Strings;
// use self::texts::Texts;
use self::values::Values;

pub use self::strings::StringId;
pub use self::values::ValueId;
pub use self::conditions::CondId;

mod paths;
mod storage;
mod strings;
mod conditions;
// mod texts;
mod values;

// -----------------------------------------------------------------------------
//   - Constants -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Constants {
    strings: Strings,
    values: Values,
    conditions: Conditions,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            strings: Strings::empty(),
            values: Values::empty(),
            conditions: Conditions::empty(),
        }
    }

    pub(crate) fn store_string(&mut self, string: impl Into<String>) -> StringId {
        self.strings.push(string.into())
    }

    pub fn store_value(&mut self, value: ScopeValue) -> ValueId {
        self.values.push(value)
    }

    pub fn store_cond(&mut self, cond: Cond) -> CondId {
        self.conditions.push(value)
    }

    pub fn lookup_string(&self, index: StringId) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    pub fn lookup_value(&self, index: ValueId) -> Option<&ScopeValue> {
        self.values.get(index)
    }

    pub fn lookup_cond(&self, index: CondId) -> Option<&Cond> {
        self.conditions.get(index)
    }
}
