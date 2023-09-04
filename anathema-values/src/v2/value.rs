use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use crate::NodeId;

use super::DIRTY_NODES;

#[derive(Debug)]
pub struct Value<T> {
    gen: usize,
    pub(crate) inner: T,
    subscribers: RefCell<Vec<NodeId>>,
}

impl<T> Value<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            gen: 0,
            subscribers: RefCell::new(vec![]),
        }
    }

    pub fn subscribe(&self, subscriber: NodeId) {
        self.subscribers.borrow_mut().push(subscriber);
    }
}

impl<T> Deref for Value<T>
where
    for<'a> Cow<'a, str>: From<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for Value<T>
where
    for<'a> Cow<'a, str>: From<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.gen = self.gen.wrapping_add(1);

        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push(s.clone()));
        }

        &mut self.inner
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
    use crate::v2::drain_dirty_nodes;

    use super::*;

    #[test]
    fn notify_subscriber() {
        let id: NodeId = 123.into();
        let mut value = Value::new("hello world".to_string());
        value.subscribe(id.clone());
        value.push_str(", updated");

        assert_eq!(id, drain_dirty_nodes()[0]);
    }
}