use std::ops::Deref;
use std::borrow::Cow;

use crate::Path;

use super::*;

#[derive(Debug)]
pub struct List<T> {
    inner: Vec<Value<T>>,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        Self::new(vec![])
    }
     
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn new(inner: Vec<Value<T>>) -> Self {
        Self { inner }
    }

    pub fn lookup(&self, key: &Path) -> Option<Cow<'_, str>>
    where
        for<'a> &'a Value<T>: Into<Cow<'a, str>>,
    {
        let Path::Index(index) = key else { return None };
        self.inner.get(*index).map(Into::into)
    }

    pub fn lookup_state(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>>
    where
        T: State,
    {
        let Path::Composite(lhs, rhs) = key.deref() else { return None };
        let Path::Index(index) = lhs.deref() else { return None };
        self.inner.get(*index).and_then(|val| val.inner.get(rhs, node_id))
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(value: Vec<T>) -> Self {
        let inner = value.into_iter().map(Value::new).collect();
        Self { inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_list() {
        let list = List::from(vec![1, 2, 3]);
    }
}
