use std::fmt::{self, Display};

use super::Storage;
use crate::slab::SlabIndex;

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

impl SlabIndex for StringId {
    const MAX: usize = usize::MAX;

    fn as_usize(&self) -> usize {
        self.0
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index)
    }
}

impl Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sid {}>", self.0)
    }
}
