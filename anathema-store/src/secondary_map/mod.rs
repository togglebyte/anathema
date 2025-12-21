pub use basic::BasicStorage;
pub use generational::GenerationalStorage;

mod basic;
mod generational;

// -----------------------------------------------------------------------------
//   - Map storage -
// -----------------------------------------------------------------------------
pub trait MapStorage: Default {
    type Key: Copy;
    type Value;

    fn insert(&mut self, key: Self::Key, value: Self::Value);

    fn remove(&mut self, key: Self::Key) -> Option<Self::Value>;

    fn remove_if<F>(&mut self, key: Self::Key, f: F) -> Option<Self::Value>
    where
        F: Fn(&Self::Value) -> bool,
    {
        let value = self.get(key)?;
        if f(value) {
            return self.remove(key);
        }

        None
    }

    fn get(&self, key: Self::Key) -> Option<&Self::Value>;

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;

    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)>;

    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)>;
}

// -----------------------------------------------------------------------------
//   - Map -
// -----------------------------------------------------------------------------

/// A secondary map holds values associated
/// with a key belonging to a [`GenSlab`].
///
/// ```
/// use anathema_store::gen_key;
/// use anathema_store::slab::{Slab, Generational};
/// use anathema_store::secondary_map::{GenerationalStorage, SecondaryMap};
///
/// gen_key!(Key);
///
/// let mut names = Generational::<Key, _>::empty();
/// let lilly = names.insert("Lilly");
///
/// let mut favourite_foods = SecondaryMap::<GenerationalStorage<Key, _>>::empty();
/// favourite_foods.insert(lilly, "apple");
///
/// assert_eq!("apple", favourite_foods.remove(lilly).unwrap());
/// ```
#[derive(Debug)]
pub struct SecondaryMap<S>(S)
where
    S: MapStorage;

impl<S, K, V> SecondaryMap<S>
where
    S: MapStorage<Key = K, Value = V>,
    S::Key: Copy,
{
    /// Create a an empty instance of a secondary map
    pub fn empty() -> Self {
        Self(S::default())
    }

    /// Insert a value into the map.
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(key, value);
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
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.0.iter()
    }

    /// Iterate over keys and values
    pub fn for_each(&mut self, f: impl Fn(K, &mut V)) {
        _ = self.0.iter_mut().map(|(k, v)| f(k, v));
    }
}

impl<S: MapStorage> std::ops::Index<S::Key> for SecondaryMap<S>
where
    S::Key: std::fmt::Debug,
{
    type Output = S::Value;

    fn index(&self, index: S::Key) -> &Self::Output {
        match self.get(index) {
            Some(val) => val,
            None => panic!("invalid key: {:?}", index),
        }
    }
}

impl<S: MapStorage> std::ops::IndexMut<S::Key> for SecondaryMap<S>
where
    S::Key: std::fmt::Debug,
{
    fn index_mut(&mut self, index: S::Key) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(val) => val,
            None => panic!("invalid key: {:?}", index),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::slab::Basic;

    #[test]
    fn insert_and_get() {
        let mut map = SecondaryMap::<BasicStorage<usize, u32>>::empty();
        // map.booh(0usize, 1);
        // assert_eq!(*map.get(0usize).unwrap(), 1);
    }

    #[test]
    fn remove_if_cond() {
        // let mut map = SecondaryMap::empty();
        // map.insert(0usize, 1);
        // map.insert(1usize, 2);
        // map.insert(2usize, 4);

        // for i in 0usize..3 {
        //     map.remove_if(i, |val| *val % 2 == 0);
        // }
    }
}
