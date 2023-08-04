use std::marker::PhantomData;
use std::hash::Hash;

use flume::Sender;

use crate::hashmap::HashMap;
use crate::{Value, ValueRef};

// We have a list of values
// We have a list of nodes
//
// We subscribe a node to one or more value.
//
// When a node is removed, all the *value* references (pointer / index, whatever) to that node needs
// to be removed.
//
// When a value changes, all the nodes that are subscribing to that value needs to be accessed
//
// When a value is removed, all the nodes subscribing to that value needs to be disconnected from
// that value
//
//
// Value -> [Node]
// Node -> [Value]
//
// Scenario setup:
//
// Value X is added.
// Node 1 and Node 2 subscribe to Value X
//
// Scenario 1: the change
// Value X change: we now need to notify Node 1 and 2
//
// Scenario 2: the node removal
// Node 2 is removed, we now need to remove any association between Value X and node 2
//
// Scenario 3: The value removal
// Value X is removed, we now need to rmeove any association between Value X and all it's nodes

#[derive(Debug)]
struct Subscribers<K, V> {
    subscribers: HashMap<K, Vec<ValueRef<V>>>,
    values: HashMap<ValueRef<V>, Vec<K>>,
}

impl<K, V> Subscribers<K, V>
where
    K: Eq + Hash + Clone,
    V: Eq + Hash + Clone + Copy,
{
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    fn subscribe_to_value(&mut self, sub: K, value: ValueRef<V>) {
        let values = self.subscribers.entry(sub.clone()).or_default();
        let subs = self.values.entry(value).or_default();
        values.push(value);
        subs.push(sub);
    }

    fn unsubscribe(&mut self, sub: K) {
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

    fn remove_value(&mut self, value: ValueRef<V>) {
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

    fn by_value(&self, value: ValueRef<V>) -> Option<&[K]> {
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

    fn setup() -> Subscribers<Sub, u32> {
        let mut subs = Subscribers::empty();
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
