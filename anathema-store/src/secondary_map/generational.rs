use super::MapStorage;
use crate::slab::Key;

// -----------------------------------------------------------------------------
//   - Entry -
// -----------------------------------------------------------------------------

#[derive(Debug)]
enum Entry<K, V> {
    Occupied(K, V),
    Vacant,
}

impl<K, V> Default for Entry<K, V> {
    fn default() -> Self {
        Self::Vacant
    }
}

impl<K: Copy, V> Entry<K, V> {
    fn try_value(&self) -> Option<(K, &V)> {
        match self {
            Entry::Occupied(key, value) => Some((*key, value)),
            Entry::Vacant => None,
        }
    }

    fn try_value_mut(&mut self) -> Option<(K, &mut V)> {
        match self {
            Entry::Occupied(key, value) => Some((*key, value)),
            Entry::Vacant => None,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Generational storage -
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct GenerationalStorage<K, V> {
    inner: Vec<Entry<K, V>>,
}

impl<K, V> Default for GenerationalStorage<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> GenerationalStorage<K, V> {
    pub const fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl<K, V> MapStorage for GenerationalStorage<K, V>
where
    K: Copy,
    K: From<Key>,
    K: PartialEq,
    Key: From<K>,
{
    type Key = K;
    type Value = V;

    fn insert(&mut self, key: Self::Key, value: Self::Value) {
        let idx = Key::from(key).index();
        if idx >= self.inner.len() {
            self.inner.resize_with(idx + 1, || Entry::default());
        }

        self.inner[idx] = Entry::Occupied(key, value);
    }

    fn remove(&mut self, key: Self::Key) -> Option<Self::Value> {
        let idx = Key::from(key).index();

        let mut old_value = Entry::Vacant;
        std::mem::swap(&mut old_value, &mut self.inner[idx]);
        match old_value {
            Entry::Occupied(value_key, value) if key == value_key => Some(value),
            Entry::Vacant | Entry::Occupied(..) => None,
        }
    }

    fn get(&self, key: Self::Key) -> Option<&Self::Value> {
        let idx = Key::from(key).index();
        self.inner.get(idx).and_then(Entry::try_value).map(|(_, val)| val)
    }

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value> {
        let idx = Key::from(key).index();
        self.inner
            .get_mut(idx)
            .and_then(Entry::try_value_mut)
            .map(|(_, val)| val)
    }

    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)> {
        self.inner.iter().filter_map(Entry::try_value)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)> {
        self.inner.iter_mut().filter_map(Entry::try_value_mut)
    }
}
