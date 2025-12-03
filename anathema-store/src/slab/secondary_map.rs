use std::marker::PhantomData;

use super::{Generational, Index};
use crate::slab::{Slab, Key, SlabIndex, SlabKey};

/// A secondary map holds values associated
/// with a key belonging to a [`GenSlab`].
///
///
/// ```
/// use anathema_store::slab::{GenSlab, Key, SecondaryMap};
///
/// let mut names = GenSlab::<Key, _>::empty();
/// let lilly = names.insert("Lilly");
///
/// let mut favourite_foods = SecondaryMap::empty();
/// favourite_foods.insert(lilly, "apple");
///
/// assert_eq!("apple", favourite_foods.remove(lilly));
/// ```
#[derive(Debug)]
pub struct SecondaryMap<S, K, V>(Generational<K, V>)
where
    S: Slab<Key = K, Value = V>;

impl<S, K, V> SecondaryMap<K, V>
where
    S: Slab<Key = K, Value = V>,
{
    /// Create a an empty instance of a secondary map
    pub fn empty() -> Self {
        Self(S::empty())
    }

    /// Insert a value into the map.
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert_at(key, value);
    }

    /// Get a reference to a value in the map
    pub fn get(&self, key: K) -> Option<&V> {
        self.0.get(key)
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.0.get_mut(key)
    }

    /// Remove a value from the map
    pub fn remove(&mut self, key: K) -> Option<V> {
        self.0.remove(key)
    }

    /// Try to remove a value from the map
    pub fn remove_if<F>(&mut self, key: K, f: F) -> Option<V>
    where
        F: Fn(&V) -> bool,
    {
        self.0.remove_if(key.into(), f)
    }

    /// Produce an iterator over the values in the secondary map
    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.0.iter_values()
    }

    /// Iterate over keys and values
    pub fn for_each(&mut self, f: impl Fn(K, &mut V)) {
        self.0.iter_mut().map(|(k, v)| f(k, v));
    }
}

impl<K, V> std::ops::Index<K> for SecondaryMap<K, V>
where
    K: SlabIndex + std::fmt::Debug,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        match self.get(index) {
            Some(val) => val,
            None => panic!("invalid key: {:?}", index),
        }
    }
}

impl<K, V> std::ops::IndexMut<K> for SecondaryMap<K, V>
where
    K: SlabIndex + std::fmt::Debug,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(val) => val,
            None => panic!("invalid key: {:?}", index),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut map = SecondaryMap::empty();
        map.insert(0usize, 1);
        assert_eq!(*map.get(0usize).unwrap(), 1);
    }

    #[test]
    fn remove_if_cond() {
        let mut map = SecondaryMap::empty();
        map.insert(0usize, 1);
        map.insert(1usize, 2);
        map.insert(2usize, 4);

        for i in 0usize..3 {
            map.remove_if(i, |val| *val % 2 == 0);
        }
    }
}
