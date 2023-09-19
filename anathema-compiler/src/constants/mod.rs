use anathema_values::{Path, ScopeValue};
pub(crate) use storage::Storage;

pub use self::conditions::CondId;
use self::conditions::Conditions;
pub use self::strings::StringId;
use self::strings::Strings;
pub use self::values::ValueId;
use self::values::Values;
use crate::parsing::parser::Cond;

mod conditions;
mod paths;
mod storage;
mod strings;
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
        self.conditions.push(cond)
    }

    pub fn lookup_string(&self, index: StringId) -> &str {
        self.strings.get(index).map(String::as_str).expect("consts have been modified, this is a bug with Anathema, file a bug report please")
    }

    pub fn lookup_value(&self, index: ValueId) -> &ScopeValue {
        self.values.get(index).expect("consts have been modified, this is a bug with Anathema, file a bug report please")
    }
}
