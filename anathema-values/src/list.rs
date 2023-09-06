use std::borrow::Cow;
use std::ops::Deref;

use super::*;
use crate::Path;

#[derive(Debug)]
pub struct List<T> {
    inner: Vec<Value<T>>,
    subscribers: RefCell<Vec<NodeId>>,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn new(inner: Vec<Value<T>>) -> Self {
        Self {
            inner,
            subscribers: RefCell::new(vec![]),
        }
    }

    pub fn subscribe(&self, node_id: NodeId) {
        self.subscribers.borrow_mut().push(node_id);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
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
        let Path::Composite(lhs, rhs) = key.deref() else {
            return None;
        };
        let Path::Index(index) = lhs.deref() else {
            return None;
        };
        self.inner
            .get(*index)
            .and_then(|val| val.inner.get(rhs, Some(node_id)))
    }

    pub fn pop(&mut self) -> Option<Value<T>> {
        let ret = self.inner.pop();
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Remove(self.inner.len()))));
        }
        ret
    }

    pub fn remove(&mut self, index: usize) -> Value<T> {
        panic!()
        // self.inner.remove(index)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        // self.inner.swap(a, b)
        panic!()
    }

    pub fn push(&mut self, value: Value<T>) {
        // self.inner.push(value)
        panic!()
    }

    pub fn insert(&mut self, index: usize, value: Value<T>) {
        // self.inner.insert(index, value)
        panic!()
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(value: Vec<T>) -> Self {
        let inner = value.into_iter().map(Value::new).collect();
        Self::new(inner)
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
