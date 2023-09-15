use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use super::DIRTY_NODES;
use crate::scope::StaticValue;
use crate::NodeId;

#[derive(Debug, PartialEq)]
pub enum Change {
    Update,
    Add,
    Remove(usize),
}

#[derive(Debug, Default)]
pub struct Value<T> {
    pub(crate) inner: T,
    subscribers: RefCell<HashSet<NodeId>>,
}

impl<T> Value<T> {
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

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Update)));
        }

        &mut self.inner
    }
}

impl<'a> From<&'a Value<String>> for Cow<'a, StaticValue> {
    fn from(value: &'a Value<String>) -> Self {
        Cow::Owned(StaticValue::Str(value.inner.as_str().into()))
    }
}

impl<'a> From<&'a Value<String>> for Cow<'a, str> {
    fn from(value: &'a Value<String>) -> Self {
        Cow::Borrowed(&value.inner)
    }
}

impl<'a> From<&'a Value<usize>> for Cow<'a, str> {
    fn from(value: &'a Value<usize>) -> Self {
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
        let mut value = Value::new("hello world".to_string());
        value.subscribe(id.clone());
        value.push_str(", updated");

        assert_eq!((id, Change::Update), drain_dirty_nodes()[0]);
    }
}
