use anathema_debug::DebugWriter;
use anathema_store::slab::Slab;
use anathema_store::smallmap::SmallIndex;
use anathema_store::stack::Stack;

use super::SUBSCRIBERS;
use crate::Key;

pub type FutureValues = Stack<Subscriber>;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
#[repr(transparent)]
pub struct KeyIndex(u8);

impl KeyIndex {
    const MAX: Self = Self(3);
    const TWO: Self = Self(2);
    const ZERO: Self = Self(0);
}

impl KeyIndex {
    fn add(&mut self) {
        debug_assert!(self.0 <= Self::MAX.0);
        self.0 += 1;
    }

    fn sub(&mut self) {
        self.0 -= 1;
    }

    const fn max() -> usize {
        Self::MAX.0 as usize
    }
}

// The key associated with the value that is being subscribed to.
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct SubKey(u32);

impl From<SubKey> for usize {
    fn from(key: SubKey) -> usize {
        key.0 as usize
    }
}

impl From<usize> for SubKey {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

/// A composite key made up of a key and an index
/// into small map.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Subscriber(Key, SmallIndex);

impl Subscriber {
    pub const MAX: Self = Self(Key::MAX, SmallIndex::MAX);
    pub const ONE: Self = Self(Key::ONE, SmallIndex::ONE);
    pub const ZERO: Self = Self(Key::ZERO, SmallIndex::ZERO);

    pub fn index(&self) -> SmallIndex {
        self.1
    }

    pub fn key(&self) -> Key {
        self.0
    }
}

impl From<(Key, SmallIndex)> for Subscriber {
    fn from((key, index): (Key, SmallIndex)) -> Self {
        Self(key, index)
    }
}

impl From<Subscriber> for Key {
    fn from(Subscriber(value, _): Subscriber) -> Self {
        value
    }
}

pub(crate) struct SubscriberDebug(pub(crate) Subscriber);

impl DebugWriter for SubscriberDebug {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        writeln!(output, "<sub key {:?} | index {}>", self.0 .0, usize::from(self.0 .1))
    }
}

/// Contains zero, one or more subscribers associated with a value.
#[derive(Debug, Clone, PartialEq)]
pub enum Subscribers {
    Empty,
    One(Subscriber),
    Arr([Subscriber; KeyIndex::max()], KeyIndex),
    Heap(Vec<Subscriber>),
}

impl Subscribers {
    /// Apply `F` to each subscriber
    pub fn with<F>(&self, f: F)
    where
        F: FnMut(Subscriber),
    {
        self.iter().for_each(f);
    }

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    // Insert a new subscriber.
    // If there are already `KeyIndex::max()` subscribers associated
    // with this value then this will cause heap allocations.
    fn insert(&mut self, sub: Subscriber) {
        match self {
            Self::Empty => *self = Self::One(sub),
            Self::One(key) => *self = Self::Arr([*key, sub, Subscriber::MAX], KeyIndex::TWO),
            Self::Arr(arr_keys, index) if *index == KeyIndex::MAX => {
                let mut keys = Vec::with_capacity(KeyIndex::max() + 1);
                keys.extend_from_slice(arr_keys);
                keys.push(sub);
                *self = Self::Heap(keys);
            }
            Self::Arr(keys, index) => {
                keys[index.0 as usize] = sub;
                index.add();
            }
            Self::Heap(keys) => keys.push(sub),
        }
    }

    // Remove a subscriber.
    // If the `Subscriber` is already using heap allocations,
    // then it will keep doing so until it reaches zero entries,
    // at which point it will become `Self::Empty`.
    fn remove(&mut self, sub: Subscriber) {
        match self {
            Self::Empty => (),
            Self::One(key) if sub == *key => *self = Self::Empty,
            Self::One(_key) => (),
            Self::Arr(arr_keys, index) => {
                let Some(pos) = arr_keys.iter().position(|k| *k == sub) else {
                    return;
                };
                arr_keys.copy_within(pos + 1.., pos);
                index.sub();
                if *index == KeyIndex::ZERO {
                    *self = Self::Empty;
                }
            }
            Self::Heap(keys) => {
                keys.iter().position(|k| *k == sub).map(|pos| keys.remove(pos));

                if keys.is_empty() {
                    *self = Self::Empty;
                }
            }
        }
    }

    /// Produce an iterator over the keys
    pub fn iter(&self) -> impl Iterator<Item = Subscriber> + '_ {
        let mut one = None;
        let mut arr = None;
        let mut heap = None;

        match self {
            Subscribers::Empty => {}
            Subscribers::One(sub) => one = Some(std::iter::once(*sub)),
            Subscribers::Arr(subs, index) => arr = Some(subs[..index.0 as usize].iter().copied()),
            Subscribers::Heap(subs) => heap = Some(subs.iter().copied()),
        };

        std::iter::from_fn(move || match self {
            Subscribers::Empty => None,
            Subscribers::One(_) => one.as_mut()?.next(),
            Subscribers::Arr(..) => arr.as_mut()?.next(),
            Subscribers::Heap(_) => heap.as_mut()?.next(),
        })
    }

    fn clear(&mut self) {
        *self = Subscribers::Empty;
    }
}

pub(super) struct SubscriberMap {
    pub(crate) inner: Slab<SubKey, Subscribers>,
}

impl SubscriberMap {
    pub(super) const fn empty() -> Self {
        Self { inner: Slab::empty() }
    }

    pub(super) fn get(&self, key: SubKey) -> Subscribers {
        self.inner.get(key).cloned().unwrap_or(Subscribers::Empty)
    }

    pub(super) fn push_empty(&mut self) -> SubKey {
        self.inner.insert(Subscribers::Empty)
    }

    pub(super) fn remove(&mut self, key: SubKey) -> Subscribers {
        self.inner.remove(key)
    }

    pub(super) fn subscribe(&mut self, key: SubKey, subscriber: Subscriber) {
        if let Some(subs) = self.inner.get_mut(key) {
            subs.insert(subscriber);
        }
    }

    pub(super) fn unsubscribe(&mut self, key: SubKey, subscriber: Subscriber) {
        if let Some(subs) = self.inner.get_mut(key) {
            subs.remove(subscriber);
        }
    }

    // Remove every subscriber but keep the entry as it is owned by a value.
    pub(super) fn clear_subscribers(&mut self) {
        for (_, subs) in self.inner.iter_mut() {
            subs.clear();
        }
    }

    #[cfg(test)]
    fn count(&self) -> usize {
        self.inner.iter_values().count()
    }
}

// Subscribe to a key
pub(crate) fn subscribe(sub_key: SubKey, subscriber: Subscriber) {
    SUBSCRIBERS.with_borrow_mut(|subs| subs.subscribe(sub_key, subscriber));
}

// Unsubscribe from a key
pub(crate) fn unsubscribe(sub_key: SubKey, subscriber: Subscriber) {
    SUBSCRIBERS.with_borrow_mut(|subs| subs.unsubscribe(sub_key, subscriber));
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Change, Value};

    fn count_subs() -> usize {
        SUBSCRIBERS.with_borrow(|subs| subs.count())
    }

    fn drain_changes() -> Vec<(Subscribers, Change)> {
        crate::store::CHANGES.with_borrow_mut(|changes| changes.drain().collect())
    }

    #[test]
    fn transition_from_empty_to_heap_and_back_to_empty() {
        let mut subs = SubscriberMap::empty();
        let key = subs.push_empty();

        let keys = [Subscriber::ZERO, Subscriber::MAX];

        for s in &keys {
            subs.subscribe(key, *s);
        }

        for s in &keys {
            subs.unsubscribe(key, *s);
        }
    }

    #[test]
    fn drop_value() {
        assert_eq!(count_subs(), 0);

        let subscriber = Subscriber::ZERO;
        let value = Value::new(123);
        let _value_ref = value.value_ref(subscriber);
        assert_eq!(count_subs(), 1);

        drop(value);

        assert_eq!(count_subs(), 0);

        let (sub, change) = drain_changes().remove(0);
        assert_eq!(change, Change::Dropped);
        assert_eq!(sub, Subscribers::One(subscriber));
    }
}
