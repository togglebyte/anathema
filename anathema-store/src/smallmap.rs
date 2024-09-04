use std::borrow::Borrow;

use crate::slab::Slab;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SmallIndex(u8);

impl SmallIndex {
    pub const MAX: Self = Self(u8::MAX);
    pub const ONE: Self = Self(1);
    pub const ZERO: Self = Self(0);
}

impl From<u8> for SmallIndex {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<usize> for SmallIndex {
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl From<SmallIndex> for usize {
    fn from(value: SmallIndex) -> Self {
        value.0 as usize
    }
}

/// A small map used to store a small amount of values.
/// ```
/// # use anathema_store::smallmap::*;
///
/// let mut map = SmallMap::empty();
/// map.set("a", 1);
/// map.set("b", 2);
/// let Some(1) = map.set("a", 5) else { unreachable!("we know there is a one there") };
///
/// let value = map.get("b").unwrap();
/// assert_eq!(2, *value);
///
/// let value = map.get("a").unwrap();
/// assert_eq!(5, *value);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SmallMap<K, V>(Slab<SmallIndex, (K, V)>);

impl<K, V> SmallMap<K, V>
where
    K: PartialEq,
{
    /// Create a en empty map
    pub fn empty() -> Self {
        Self(Slab::empty())
    }

    /// Set a value in the map.
    /// If there is already a value with the same key,
    /// the old value will be returned
    pub fn set(&mut self, key: K, mut value: V) -> Option<V> {
        let old_value = self.get_mut(&key);
        match old_value {
            Some(old) => {
                std::mem::swap(old, &mut value);
                Some(value)
            }
            None => {
                self.0.insert((key, value));
                None
            }
        }
    }

    pub fn insert_with<F>(&mut self, key: K, f: F) -> SmallIndex
    where
        F: FnOnce(SmallIndex) -> V,
    {
        let id = self.0.next_id();
        let value = f(id);
        assert_eq!(self.0.insert((key, value)), id);
        id
    }

    /// Get a reference to a value in the map
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        self.0.iter().find_map(|(_, (k, v))| match k.borrow() == key {
            true => Some(v),
            false => None,
        })
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        self.0.iter_mut().find_map(|(_, (k, v))| match (*k).borrow() == key {
            true => Some(v),
            false => None,
        })
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<SmallIndex>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        self.0.iter().find_map(|(i, (k, _))| match k.borrow() == key {
            true => Some(i),
            false => None,
        })
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        let idx = self.get_index(key)?;
        self.0.try_remove(idx).map(|(_, v)| v)
    }

    /// Iterate over the key-value pairs of the map.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        self.0.iter().map(|(_, (k, v))| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut K, &mut V)> + '_ {
        self.0.iter_mut().map(|(_, (k, v))| (k, v))
    }

    /// Get a value ref by the value index instead of the key
    pub fn get_with_index(&self, idx: SmallIndex) -> Option<&V> {
        self.0.get(idx).map(|(_, v)| v)
    }

    /// Get a mutable value ref by the value index instead of the key
    pub fn get_mut_with_index(&mut self, idx: SmallIndex) -> Option<&mut V> {
        self.0.get_mut(idx).map(|(_, v)| v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_twice() {
        let mut map = SmallMap::<&str, u8>::empty();
        map.set("a", 1);
        map.set("b", 2);

        assert_eq!(1, *map.get("a").unwrap());
    }

    #[test]
    fn get_and_get_mut() {
        let mut map = SmallMap::<&str, u8>::empty();
        map.set("a", 1);
        map.set("b", 2);

        *map.get_mut("a").unwrap() += 1;

        assert_eq!(2, *map.get("b").unwrap());
        assert_eq!(2, *map.get("a").unwrap());
    }

    #[test]
    fn double_set() {
        let mut map = SmallMap::<&str, u8>::empty();
        assert_eq!(None, map.set("a", 1));
        assert_eq!(Some(1), map.set("a", 2));
    }

    #[test]
    fn get_by_index() {
        let mut map = SmallMap::<&str, u8>::empty();
        map.set("a", 1);
        let idx = map.get_index("a").unwrap();
        assert_eq!(1, *map.get_with_index(idx).unwrap());
    }

    #[test]
    fn get_by_index_mut() {
        let mut map = SmallMap::<&str, u8>::empty();
        map.set("a", 1);
        let idx = map.get_index("a").unwrap();
        assert_eq!(1, *map.get_mut_with_index(idx).unwrap());
    }
}
