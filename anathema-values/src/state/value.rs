use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use crate::{NodeId, ValueRef, DIRTY_NODES, Owned};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    Update,
    Add,
    Remove(usize),
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

    pub fn subscribe(&self, subscriber: NodeId) {
        self.subscribers.borrow_mut().insert(subscriber);
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

impl<'a, T: Into<Owned> + Copy> From<&'a StateValue<T>> for ValueRef<'a> {
    fn from(value: &'a StateValue<T>) -> Self {
        ValueRef::Owned(value.inner.into())
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
