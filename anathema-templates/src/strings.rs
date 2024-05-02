use std::fmt::{self, Display};

pub type Strings = anathema_store::Storage<StringId, String>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StringId(usize);

impl From<usize> for StringId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Into<usize> for StringId {
    fn into(self) -> usize {
        self.0
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sid {}>", self.0)
    }
}
