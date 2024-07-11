use std::fmt::{self, Display};

use super::Storage;

pub struct Strings {
    inner: Storage<StringId, String, ()>,
}

impl Strings {
    pub fn empty() -> Self {
        Self {
            inner: Storage::empty(),
        }
    }

    pub fn push(&mut self, string: impl Into<String>) -> StringId {
        self.inner.push(string, ())
    }

    pub fn get_unchecked(&self, string_id: StringId) -> String {
        self.inner.get_unchecked(string_id).0.clone()
    }
}

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
