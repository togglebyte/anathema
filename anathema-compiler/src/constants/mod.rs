use anathema_values::{ValueExpr};
pub(crate) use storage::Storage;

pub use self::strings::StringId;
use self::strings::Strings;
pub use self::values::ValueId;
use self::values::Values;

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

    pub fn store_value(&mut self, value: ValueExpr) -> ValueId {
        self.values.push(value)
    }

    pub fn lookup_string(&self, index: StringId) -> &str {
        self.strings.get(index).map(String::as_str).expect(
            "consts have been modified, this is a bug with Anathema, file a bug report please",
        )
    }

    pub fn lookup_value(&self, index: ValueId) -> ValueExpr {
        self.values.get(index).cloned().expect(
            "consts have been modified, this is a bug with Anathema, file a bug report please",
        )
    }
}
