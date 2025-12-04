use std::fmt::{self, Display};

use super::Storage;

pub struct Strings {
    inner: Storage<StringId, String>,
}

impl Strings {
    pub fn empty() -> Self {
        Self {
            inner: Storage::empty(),
        }
    }

    pub fn push(&mut self, string: impl Into<String>) -> StringId {
        self.inner.insert(string)
    }

    pub fn lookup(&self, string: &str) -> Option<StringId> {
        self.inner.find_key(string)
    }

    pub fn get(&self, string_id: StringId) -> Option<&str> {
        self.inner.get(string_id).map(String::as_str)
    }

    pub fn get_unchecked(&self, string_id: StringId) -> String {
        self.inner.get_unchecked(string_id).clone()
    }

    pub fn get_ref_unchecked(&self, string_id: StringId) -> &str {
        &self.inner.get_unchecked(string_id).as_str()
    }
}

// TODO: change this to u16
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StringId(usize);

impl From<StringId> for usize {
    fn from(value: StringId) -> Self {
        value.0
    }
}

impl From<usize> for StringId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sid {}>", self.0)
    }
}
