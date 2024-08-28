use std::marker::PhantomData;

use super::{Index, Slab};

/// A secondary map holds values associated
/// with a key belonging to a [`GenSlab`].
///
///
/// ```
/// use anathema_store::slab::{GenSlab, SecondaryMap};
///
/// let mut names = GenSlab::empty();
/// let lilly = names.insert("Lilly");
///
/// let mut favourite_foods = SecondaryMap::empty();
/// favourite_foods.insert(lilly, "apple");
///
/// assert_eq!("apple", favourite_foods.remove(lilly));
/// ```
// The reason this is not using a `GenSlab`: the key size for the gen
// slab is 64bits whereas the basic slab can make do with 32 bits.
//
// This means we can store the generation (when needed) as part of the value instead.
#[derive(Debug)]
pub struct SecondaryMap<K, V>(Slab<Index, V>, PhantomData<K>);

impl<K, V> SecondaryMap<K, V>
where
    K: Into<Index>,
{
    /// Create a an empty instance of a secondary map
    pub fn empty() -> Self {
        Self(Slab::empty(), PhantomData)
    }

    /// Insert a value into the map.
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert_at(key.into(), value);
    }

    /// Get a reference to a value in the map
    pub fn get(&self, key: K) -> Option<&V> {
        self.0.get(key.into())
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.0.get_mut(key.into())
    }

    /// Remove a value from the map
    pub fn remove(&mut self, key: K) -> V {
        self.0.remove(key.into())
    }

    /// Try to remove a value from the map
    pub fn try_remove(&mut self, key: K) -> Option<V> {
        self.0.try_remove(key.into())
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
        self.0.iter().map(|(_, v)| v)
    }
}
