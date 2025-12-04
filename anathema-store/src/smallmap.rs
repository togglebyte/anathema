use std::borrow::Borrow;
use std::ops::{Index, IndexMut};

use crate::slab::{Basic, Slab};

type NumType = u16;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct SmallIndex(pub(super) NumType);


impl SmallIndex {
    pub const MAX: Self = Self(NumType::MAX);
    pub const ONE: Self = Self(1);
    pub const ZERO: Self = Self(0);
}

impl From<SmallIndex> for NumType {
    fn from(value: SmallIndex) -> Self {
        value.0
    }
}

impl From<SmallIndex> for usize {
    fn from(value: SmallIndex) -> Self {
        value.0 as Self
    }
}

impl From<usize> for SmallIndex {
    fn from(value: usize) -> Self {
        assert!(value <= NumType::MAX as usize, "value is larger than the index allows");
        Self(value as NumType)
    }
}

/// A small map used to store a small amount of values.
///
/// The `SmallMap` can store up to 256 values.
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
pub struct SmallMap<K, V>(Basic<SmallIndex, (K, V)>);

impl<K, V> SmallMap<K, V>
where
    K: PartialEq,
{
    /// Create a en empty map
    pub const fn empty() -> Self {
        Self(Basic::empty())
    }

    /// Set a value in the map.
    /// If there is already a value with the same key it will be overwritten.
    pub fn set(&mut self, key: K, mut value: V) -> SmallIndex {
        match self.get_index(&key) {
            Some(index) => {
                self.0.replace(index, (key, value));
                index
            }
            None => self.0.insert((key, value)),
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
        self.0.iter().find_map(|(_, (k, v))| k.borrow().eq(key).then_some(v))
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        self.0
            .iter_mut()
            .find_map(|(_, (k, v))| (*k).borrow().eq(key).then_some(v))
    }

    pub fn get_index<Q>(&self, key: &Q) -> Option<SmallIndex>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        self.0.iter().find_map(|(i, (k, _))| k.borrow().eq(key).then_some(i))
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: PartialEq + ?Sized,
    {
        let idx = self.get_index(key)?;
        Some(self.0.remove(idx).1)
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

impl<K, V> Index<SmallIndex> for SmallMap<K, V> {
    type Output = V;

    fn index(&self, index: SmallIndex) -> &Self::Output {
        &self.0[index].1
    }
}

impl<K, V> IndexMut<SmallIndex> for SmallMap<K, V> {
    fn index_mut(&mut self, index: SmallIndex) -> &mut Self::Output {
        &mut self.0[index].1
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
        let index_1 = map.set("a", 1);
        assert_eq!(Some(&1), map.get("a"));
        let index_2 = map.set("a", 2);
        assert_eq!(Some(&2), map.get("a"));
        assert_eq!(index_1, index_2);
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
