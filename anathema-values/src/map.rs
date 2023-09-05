use std::borrow::Cow;
use std::ops::Deref;

use super::*;
use crate::hashmap::HashMap;
use crate::Path;

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, Value<T>>,
}

impl<T> Map<T> {
    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    pub fn new(inner: HashMap<String, Value<T>>) -> Self {
        Self { inner }
    }

    pub fn lookup(&self, key: &Path) -> Option<Cow<'_, str>>
    where
        for<'a> &'a Value<T>: Into<Cow<'a, str>>,
    {
        let Path::Key(key) = key else { return None };
        self.inner.get(key).map(Into::into)
    }

    pub fn lookup_state(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>>
    where
        T: State,
    {
        let Path::Composite(lhs, rhs) = key else { return None };
        let Path::Key(key) = lhs.deref() else { return None };
        self.inner.get(key).and_then(|val| val.inner.get(rhs, Some(node_id)))
    }
}
