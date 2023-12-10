use std::fmt::Display;

use super::Storage;

// TODO: maybe not make this public?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StringId(pub usize);

impl From<usize> for StringId {
    fn from(n: usize) -> Self {
        Self(n)
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<sid {}>", self.0)
    }
}

#[derive(Debug)]
pub struct Strings(Storage<String>);

impl Strings {
    pub(crate) fn empty() -> Self {
        Self(Storage::empty())
    }

    pub(crate) fn push(&mut self, string: String) -> StringId {
        StringId(self.0.push(string))
    }

    pub(crate) fn get(&self, index: StringId) -> Option<&String> {
        self.0.get(index.0)
    }
}
