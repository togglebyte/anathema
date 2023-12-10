use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use crate::{Collection, List, NodeId, Owned, Path, State, ValueRef, DIRTY_NODES};

// TODO: Can we make this `Copy` as well?
//       This depends if `RemoveKey` is required here or not.
//       TB 2023-11-11
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Update,
    Push,
    InsertIndex(usize),
    InsertKey(String),
    RemoveIndex(usize),
    RemoveKey(String),
}

#[derive(Debug, Default)]
pub struct StateValue<T> {
    pub(crate) inner: T,
    subscribers: RefCell<HashSet<NodeId>>,
}

impl<T> StateValue<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            subscribers: RefCell::new(HashSet::new()),
        }
    }

    #[doc(hidden)]
    pub fn get(&self, _: &Path, _: Option<&NodeId>) -> ValueRef<'_> {
        ValueRef::Empty
    }

    pub fn subscribe(&self, subscriber: NodeId) {
        self.subscribers.borrow_mut().insert(subscriber);
    }
}

impl<T> StateValue<T>
where
    for<'b> &'b T: Into<ValueRef<'b>>,
{
    pub fn get_value(&self, node_id: Option<&NodeId>) -> ValueRef<'_> {
        if let Some(node_id) = node_id.cloned() {
            self.subscribe(node_id);
        }
        (&self.inner).into()
    }
}

impl<T> Deref for StateValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for StateValue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Update)));
        }

        &mut self.inner
    }
}

impl<'a> From<&'a StateValue<String>> for ValueRef<'a> {
    fn from(value: &'a StateValue<String>) -> Self {
        ValueRef::Str(value.inner.as_str())
    }
}

impl<'a, T> From<&'a StateValue<T>> for ValueRef<'a>
where
    Owned: From<&'a T>,
{
    fn from(value: &'a StateValue<T>) -> Self {
        ValueRef::Owned((&value.inner).into())
    }
}

impl<T: State> State for StateValue<T> {
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        self.inner.get(key, node_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::drain_dirty_nodes;

    #[test]
    fn notify_subscriber() {
        let id: NodeId = 123.into();
        let mut value = StateValue::new("hello world".to_string());
        value.subscribe(id.clone());
        value.push_str(", updated");

        assert_eq!((id, Change::Update), drain_dirty_nodes()[0]);
    }
}
