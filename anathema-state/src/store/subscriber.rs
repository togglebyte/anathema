use anathema_store::slab::{Slab, SlabIndex};
use anathema_store::smallmap::SmallIndex;

use super::SUBSCRIBERS;
use crate::Key;

/// Store sub keys.
/// This is used by any type that needs to track any or all the
/// values that it subscribes to.
#[derive(Debug, Default)]
pub enum SubTo {
    #[default]
    Zero,
    One(SubKey),
    Two(SubKey, SubKey),
    Three(SubKey, SubKey, SubKey),
    Four(SubKey, SubKey, SubKey, SubKey),
    Many(Vec<SubKey>),
}

impl SubTo {
    pub fn empty() -> Self {
        Self::Zero
    }

    pub fn push(&mut self, key: SubKey) {
        let this = std::mem::take(self);
        *self = match this {
            Self::Zero => Self::One(key),
            Self::One(key1) => Self::Two(key1, key),
            Self::Two(key1, key2) => Self::Three(key1, key2, key),
            Self::Three(key1, key2, key3) => Self::Four(key1, key2, key3, key),
            Self::Four(key1, key2, key3, key4) => Self::Many(vec![key1, key2, key3, key4]),
            Self::Many(mut keys) => {
                keys.push(key);
                Self::Many(keys)
            }
        }
    }

    pub fn unsubscribe(&mut self, sub: Subscriber) {
        match std::mem::take(self) {
            SubTo::Zero => return,
            SubTo::One(key) => {
                unsubscribe(key, sub);
            }
            SubTo::Two(key1, key2) => {
                unsubscribe(key1, sub);
                unsubscribe(key2, sub);
            }
            SubTo::Three(key1, key2, key3) => {
                unsubscribe(key1, sub);
                unsubscribe(key2, sub);
                unsubscribe(key3, sub);
            }
            SubTo::Four(key1, key2, key3, key4) => {
                unsubscribe(key1, sub);
                unsubscribe(key2, sub);
                unsubscribe(key3, sub);
                unsubscribe(key4, sub);
            }
            SubTo::Many(vec) => vec.into_iter().for_each(|key| unsubscribe(key, sub)),
        }
    }

    // TODO: clean this up, it's gross
    pub fn remove(&mut self, sub_key: SubKey) {
        match self {
            SubTo::Zero => return,
            SubTo::One(key) if *key == sub_key => *self = SubTo::Zero,
            SubTo::Two(key1, key2) if *key1 == sub_key => *self = SubTo::One(*key2),
            SubTo::Two(key1, key2) if *key2 == sub_key => *self = SubTo::One(*key1),
            SubTo::Three(key1, key2, key3) => {
                if sub_key == *key1 {
                    *self = SubTo::Two(*key2, *key3);
                    return;
                }

                if sub_key == *key2 {
                    *self = SubTo::Two(*key1, *key3);
                    return;
                }

                if sub_key == *key3 {
                    *self = SubTo::Two(*key1, *key2);
                    return;
                }
            }
            SubTo::Four(key1, key2, key3, key4) => {
                if sub_key == *key1 {
                    *self = SubTo::Three(*key2, *key3, *key4);
                    return;
                }

                if sub_key == *key2 {
                    *self = SubTo::Three(*key1, *key3, *key4);
                    return;
                }

                if sub_key == *key3 {
                    *self = SubTo::Three(*key1, *key2, *key4);
                    return;
                }

                if sub_key == *key4 {
                    *self = SubTo::Three(*key1, *key2, *key3);
                    return;
                }
            }
            SubTo::Many(vec) => drop(vec.retain(|key| *key != sub_key)),
            _ => {}
        }
    }
}

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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct SubKey(u32);

impl SlabIndex for SubKey {
    const MAX: usize = u32::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
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

    #[cfg(test)]
    fn len(&self) -> usize {
        match self {
            Subscribers::Empty => 0,
            Subscribers::One(_) => 1,
            Subscribers::Arr(_, index) => index.0 as usize,
            Subscribers::Heap(vec) => vec.len(),
        }
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
        self.inner.iter_values().map(|s| s.len()).sum()
    }
}

// Subscribe to a key
pub(crate) fn subscribe(sub_key: SubKey, subscriber: Subscriber) {
    anathema_debug::debug_to_file!("subscribed to sub key {:?} | subscriber: {subscriber:?}", sub_key);
    SUBSCRIBERS.with_borrow_mut(|subs| subs.subscribe(sub_key, subscriber));
}

// Unsubscribe from a key
pub(crate) fn unsubscribe(sub_key: SubKey, subscriber: Subscriber) {
    anathema_debug::debug_to_file!("unsubscribed with sub key {:?} | subscriber {subscriber:?}", sub_key);
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
