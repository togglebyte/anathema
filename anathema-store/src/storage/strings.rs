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

    pub fn lookup(&self, string: &str) -> Option<StringId> {
        self.inner.iter().find_map(|(i, (k, _))| match k == string {
            true => Some(i),
            false => None,
        })
    }

    pub fn get(&self, string_id: StringId) -> Option<&str> {
        self.inner.get(string_id).map(|(k, _v)| k.as_str())
    }

    pub fn get_unchecked(&self, string_id: StringId) -> String {
        self.inner.get_unchecked(string_id).0.clone()
    }

    pub fn get_ref_unchecked(&self, string_id: StringId) -> &str {
        &self.inner.get_unchecked(string_id).0
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
