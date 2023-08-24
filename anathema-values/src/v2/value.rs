use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Value<T, S> {
    gen: usize,
    pub(crate) inner: T,
    subscribers: Vec<S>
}

impl<T, S> Value<T, S> {
    pub fn new(inner: T) -> Self {
        Self { inner, gen: 0, subscribers: vec![] }
    }

    pub fn subscribe(&mut self, subscriber: S) {
        self.subscribers.push(subscriber);
    }
}

impl<T, S> Deref for Value<T, S>
where
    for<'a> Cow<'a, str>: From<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T, S> DerefMut for Value<T, S>
where
    for<'a> Cow<'a, str>: From<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.gen = self.gen.wrapping_add(1);
        // Notify changes
        for s in &mut self.subscribers {
            // s.notify();
        }
        &mut self.inner
    }
}

impl<'a, S> From<&'a Value<String, S>> for Cow<'a, str> {
    fn from(value: &'a Value<String, S>) -> Self {
        Cow::Borrowed(&value.inner)
    }
}

impl<'a, S> From<&'a Value<usize, S>> for Cow<'a, str> {
    fn from(value: &'a Value<usize, S>) -> Self {
        Cow::Owned(value.inner.to_string())
    }
}
