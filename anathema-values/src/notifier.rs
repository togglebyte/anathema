use std::marker::PhantomData;
use std::hash::Hash;

use flume::Sender;

use crate::hashmap::HashMap;
use crate::{Value, ValueRef};

pub trait Listen {
    type Value;
    type Key: Eq + Hash + Clone;

    fn subscribe(value: ValueRef<Value<Self::Value>>, key: Self::Key);
}

#[derive(Debug)]
pub struct Listeners<K, V> {
    subscribers: HashMap<K, Vec<ValueRef<Value<V>>>>,
    values: HashMap<ValueRef<Value<V>>, Vec<K>>,
}

impl<K, V> Listeners<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe_to_value(&mut self, sub: K, value: ValueRef<Value<V>>) {
        let values = self.subscribers.entry(sub.clone()).or_default();
        let subs = self.values.entry(value).or_default();
        values.push(value);
        subs.push(sub);
    }

    pub fn unsubscribe(&mut self, sub: K) {
        let values = self.subscribers.remove(&sub).unwrap_or_default();
        for value in &values {
            let Some(subs) = self.values.get_mut(value) else {
                continue;
            };
            if let Some(pos) = subs.iter().position(|s| sub.eq(s)) {
                subs.remove(pos);
            }
            if subs.is_empty() {
                self.values.remove(value);
            }
        }
    }

    pub fn remove_value(&mut self, value: ValueRef<Value<V>>) {
        let nodes = self.values.remove(&value).unwrap_or_default();
        for node in &nodes {
            let Some(values) = self.subscribers.get_mut(node) else {
                continue;
            };
            if let Some(pos) = values.iter().position(|val| value.eq(val)) {
                values.remove(pos);
            }
            if values.is_empty() {
                self.subscribers.remove(node);
            }
        }
    }

    pub fn by_value(&self, value: ValueRef<Value<V>>) -> Option<&[K]> {
        self.values.get(&value).map(Vec::as_slice)
    }
}

pub(crate) struct Change<T>(ValueRef<Value<T>>, Action<T>);

pub struct Notifier<T> {
    sender: Sender<Change<T>>,
}

impl<T> Notifier<T> {
    pub(crate) fn new(sender: Sender<Change<T>>) -> Self {
        Self { 
            sender 
        }
    }

    pub fn notify(&self, value_ref: ValueRef<Value<T>>, change: Action<T>) {
        let banana = Change(value_ref, change);
        self.sender.send(banana);
    }
}

pub enum Action<T> {
    Modified,
    Add,
    Remove(ValueRef<T>),
    Swap(ValueRef<T>, ValueRef<T>),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ValueRef;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    struct Sub(usize);

    fn setup() -> Listeners<Sub, u32> {
        let mut subs = Listeners::empty();
        subs.subscribe_to_value(Sub(0), ValueRef::new(0, 0));
        subs.subscribe_to_value(Sub(0), ValueRef::new(1, 0));
        subs.subscribe_to_value(Sub(0), ValueRef::new(2, 0));

        subs.subscribe_to_value(Sub(1), ValueRef::new(1, 0));
        subs.subscribe_to_value(Sub(1), ValueRef::new(2, 0));

        subs.subscribe_to_value(Sub(2), ValueRef::new(2, 0));
        subs
    }

    #[test]
    fn get_subscribers() {
        let subs = setup();
        let value = ValueRef::new(2, 0);
        let values = subs.by_value(value).unwrap();

        assert_eq!(*&values[0], Sub(0));
        assert_eq!(*&values[1], Sub(1));
        assert_eq!(*&values[2], Sub(2));
    }

    #[test]
    fn remove_value() {
        let mut subs = setup();
        let value = ValueRef::new(2, 0);
        subs.remove_value(value);
        assert!(subs.by_value(value).is_none());
        assert!(subs.subscribers.get(&Sub(2)).is_none());
    }

    #[test]
    fn unsub() {
        let mut subs = setup();
        let value = ValueRef::new(2, 0);
        subs.unsubscribe(Sub(2));
        let values = subs.by_value(value).unwrap();
        assert_eq!(values.len(), 2);
    }
}
