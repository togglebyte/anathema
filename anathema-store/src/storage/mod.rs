use std::ops::Index;

use crate::slab::{Basic, Slab};

pub mod strings;

/// Simple storage backed by a slab, prevents duplicate values
/// and associate values with keys
pub struct Storage<K, V>(Basic<K, V>);

impl<K, V> Storage<K, V>
where
    K: Copy,
    K: From<usize>,
    K: PartialEq,
    usize: From<K>,
{
    /// Create an empty store
    pub const fn empty() -> Self {
        Self(Basic::empty())
    }

    /// De-duplicate values.
    /// If the key already exist, just return the index.
    ///
    /// # Note
    ///
    /// This will not overwrite the existing value.
    #[must_use]
    pub fn push(&mut self, value: impl Into<V>) -> K
    where
        V: PartialEq,
    {
        let value = value.into();

        if let Some(key) = self.0.iter().find_map(|(k, v)| value.eq(v).then_some(k)) {
            return key;
        }

        self.0.insert(value)
    }

    /// Insert a key and a value.
    /// If the key already exists the value will be overwritten
    #[must_use]
    pub fn insert(&mut self, value: impl Into<V>) -> K
    where
        V: PartialEq,
    {
        let value = value.into();
        if let Some(k) = self.0.iter().find_map(|(k, v)| value.eq(v).then_some(k)) {
            return k;
        }
        self.push(value)
    }

    /// Get a reference by index
    pub fn get(&self, key: K) -> Option<&V> {
        self.0.get(key)
    }

    /// Get a mutable reference by index
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.0.get_mut(key)
    }

    /// Get a value by index assuming the value exists.
    ///
    /// # Panics
    ///
    /// If the value doesn't exist
    pub fn get_unchecked(&self, key: K) -> &V {
        self.0.get(key).expect("missing value")
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        self.0.try_remove(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.0.iter_mut()
    }

    pub fn find_key<Q>(&self, value: &Q) -> Option<K>
    where
        V: std::borrow::Borrow<Q>,
        Q: ?Sized,
        Q: PartialEq,
    {
        self.0.iter().find_map(|(k, v)| (v.borrow() == value).then_some(k))
    }
}

impl<K, V> Index<K> for Storage<K, V>
where
    usize: From<K>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.0[index]
    }
}
