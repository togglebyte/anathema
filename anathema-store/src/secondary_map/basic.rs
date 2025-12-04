use std::marker::PhantomData;

use super::MapStorage;

// -----------------------------------------------------------------------------
//   - Basic storage -
//   Used by secondary maps only
// -----------------------------------------------------------------------------
#[derive(Debug)]
enum Entry<T> {
    Occupied(T),
    Vacant,
}

impl<T> Entry<T> {
    fn try_value(&self) -> Option<&T> {
        match self {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant => None,
        }
    }

    fn try_value_mut(&mut self) -> Option<&mut T> {
        match self {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant => None,
        }
    }
}

impl<T> Default for Entry<T> {
    fn default() -> Self {
        Self::Vacant
    }
}

pub struct BasicStorage<K, V> {
    inner: Vec<Entry<V>>,
    p: PhantomData<K>,
}

impl<K, V> Default for BasicStorage<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> BasicStorage<K, V> {
    pub const fn new() -> Self {
        Self {
            inner: vec![],
            p: PhantomData,
        }
    }
}

impl<K, V> MapStorage for BasicStorage<K, V>
where
    K: Copy,
    K: From<usize>,
    usize: From<K>,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        let idx = usize::from(key);
        if idx >= self.inner.len() {
            self.inner.resize_with(idx, || Entry::default());
        }

        self.inner[idx] = Entry::Occupied(value);
    }

    fn remove(&mut self, key: Self::Key) -> Option<Self::Value> {
        let idx = usize::from(key);

        let mut old_value = Entry::Vacant;
        std::mem::swap(&mut old_value, &mut self.inner[idx]);
        match old_value {
            Entry::Vacant => None,
            Entry::Occupied(val) => Some(val),
        }
    }

    fn get(&self, key: Self::Key) -> Option<&Self::Value> {
        let idx = usize::from(key);
        self.inner.get(idx).and_then(Entry::try_value)
    }

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value> {
        let idx = usize::from(key);
        self.inner.get_mut(idx).and_then(Entry::try_value_mut)
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)> {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| entry.try_value().map(|val| (idx, val)))
            .map(|(idx, value)| (Self::Key::from(idx), value))
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)> {
        self.inner
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, entry)| entry.try_value_mut().map(|val| (idx, val)))
            .map(|(idx, value)| (Self::Key::from(idx), value))
    }
}
