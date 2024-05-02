use super::{GenSlab, Key};

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
/// assert_eq!("apple", favourite_foods.remove(lilly).unwrap());
/// ```
pub struct SecondaryMap<T>(GenSlab<T>);

impl<T> SecondaryMap<T> {
    /// Create a an empty instance of a secondary map
    pub fn empty() -> Self {
        Self(GenSlab::empty())
    }

    /// Insert a value into the map.
    pub fn insert(&mut self, key: Key, value: T) {
        self.0.insert_at(key, value);
    }

    /// Get a reference to a value in the map
    pub fn get(&self, key: Key) -> Option<&T> {
        self.0.get(key)
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        self.0.get_mut(key)
    }

    /// Remove a value from the map
    pub fn remove(&mut self, key: Key) -> Option<T> {
        self.0.remove(key)
    }

    /// Produce an iterator over the values in the secondary map
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}
