use std::fmt::{self, Display};

pub use anathema_store::storage::strings::StringId;
use anathema_store::storage::strings::Strings as StringStore;

static CHILDREN: &str = "children";

/// This differs from the storage `Strings` only on account 
/// of having something akin to a constant for children
pub struct Strings {
    inner: StringStore,
    children: StringId,
}

impl Strings {
    pub fn new() -> Self {
        let mut inner = StringStore::empty();
        let children = inner.push(CHILDREN);

        Self { inner, children }
    }

    pub fn children(&self) -> StringId {
        self.children
    }

    pub(crate) fn push(&mut self, string: impl Into<String>) -> StringId {
        self.inner.push(string)
    }

    pub(crate) fn get_unchecked(&self, string_id: StringId) -> String {
        self.inner.get_unchecked(string_id)
    }

    pub fn get_ref_unchecked(&self, string_id: StringId) -> &str {
        self.inner.get_ref_unchecked(string_id)
    }

    pub fn lookup(&self, string: &str) -> Option<StringId> {
        self.inner.lookup(string)
    }
}
