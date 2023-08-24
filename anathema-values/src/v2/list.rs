use std::ops::Deref;
use std::borrow::Cow;

use crate::Path;

use super::*;

#[derive(Debug)]
pub struct List<T, S> {
    inner: Vec<Value<T, S>>,
}

impl<T, S> List<T, S> {
    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn new(inner: Vec<Value<T, S>>) -> Self {
        Self { inner }
    }

    pub fn lookup(&self, key: &Path) -> Option<Cow<'_, str>>
    where
        for<'a> &'a Value<T, S>: Into<Cow<'a, str>>,
    {
        let Path::Index(index) = key else { return None };
        self.inner.get(*index).map(Into::into)
    }

    pub fn lookup_state(&self, key: &Path) -> Option<Cow<'_, str>>
    where
        T: State,
    {
        let Path::Composite(lhs, rhs) = key.deref() else { return None };
        let Path::Index(index) = lhs.deref() else { return None };
        self.inner.get(*index).and_then(|val| val.inner.get(rhs))
    }
}
