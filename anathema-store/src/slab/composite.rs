use crate::slab::{GenSlab, Key, SecondaryMap};

/// A composite store made up of a slab and a secondary map that holds a `Vec<U>`.
pub struct Composite<T, U> {
    slab: GenSlab<T>,
    reverse: SecondaryMap<Key, Vec<U>>,
}

impl<T, U> Composite<T, U> {
    /// Get a reference to a value
    pub fn get(&self, key: Key) -> Option<&T> {
        self.slab.get(key)
    }

    /// Get a mutable reference to a value
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        self.slab.get_mut(key)
    }

    /// Insert a new value.
    pub fn insert(&mut self, value: T) -> Key {
        let key = self.slab.insert(value);
        self.reverse.insert(key, vec![]);
        key
    }

    /// Associate a secondary value with the key
    pub fn associate_with(&mut self, key: Key, assoc: U) {
        let Some(vec) = self.reverse.get_mut(key) else { return };
        vec.push(assoc);
    }

    /// Removing a value returns all the associted values
    pub fn delete(&mut self, key: Key) -> Vec<U> {
        self.slab.remove(key);
        self.reverse.remove(key)
    }
}
