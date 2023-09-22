use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use crate::DIRTY_NODES;
use crate::scope::Value;
use crate::NodeId;

#[derive(Debug, PartialEq)]
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

impl<'a> From<&'a StateValue<String>> for Cow<'a, Value> {
    fn from(value: &'a StateValue<String>) -> Self {
        Cow::Owned(Value::Str(value.inner.as_str().into()))
    }
}

impl<'a> From<&'a StateValue<String>> for Cow<'a, str> {
    fn from(value: &'a StateValue<String>) -> Self {
        Cow::Borrowed(&value.inner)
    }
}

impl<'a> From<&'a StateValue<usize>> for Cow<'a, str> {
    fn from(value: &'a StateValue<usize>) -> Self {
        Cow::Owned(value.inner.to_string())
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
