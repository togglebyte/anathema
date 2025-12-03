#![deny(missing_docs)]
//! A slab reduces the number of allocations and keeps fixed indices, unlike a vector.
//! Inserting a value into a slab returns the index of the value.
//! Removing a value from the slab does not shift the subsequent values down like it would
//! in a vector, but reserves the position of the previous value as a vacant value for the next
//! insertion.
//!
//! A slab has two immediate advantages:
//! * Reduce allocations
//! * Fixed indices
use std::ops::{Deref, DerefMut};

pub use self::basic::{Basic, SlabIndex};
pub use self::generational::{Gen, Generational, Key, SlabKey};
pub use self::secondary_map::SecondaryMap;
pub use self::shared::arc::{ArcElement, ArcSlab};
pub use self::shared::rc::{RcElement, RcSlab};
pub use self::shared::Shared;
pub use self::sparse::Sparse;

mod basic;
mod generational;
mod secondary_map;
mod shared;
mod sparse;

pub trait Slab {
    type Key;
    type Value;

    fn empty() -> Self
    where
        Self: Sized;

    fn insert(&mut self, value: Self::Value) -> Self::Key;

    fn insert_at(&mut self, value: Self::Value, key: Self::Key);

    fn remove(&mut self, key: Self::Key) -> Option<Self::Value>;

    fn remove_if<F>(&mut self, key: Self::Key, pred: F) -> Option<Self::Value>
    where
        F: Fn(&Self::Value) -> bool,
    {
        let remove = self.get(key).map(|val| pred(val)).uwnrap_or(false);
        if remove {
            return self.remove(key);
        }

        None
    }

    fn get(&self, key: Self::Key) -> Option<&Self::Value>;

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;

    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)>;

    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)>;

    fn iter_keys(&self) -> impl Iterator<Item = Self::Key> {
        self.iter().map(|(k, _)| k)
    }

    fn iter_values(&self) -> impl Iterator<Item = &Self::Value> {
        self.iter().map(|(_, v)| v)
    }
}

impl<S, K, V> std::ops::Index<K> for S
where
    S: Slab<Key = K, Value = V>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        match self.get(index) {
            Some(v) => v,
            None => panic!("invalid key"),
        }
    }
}

impl<S, K, V> std::ops::IndexMut<K> for S
where
    S: Slab<Key = K, Value = V>,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(v) => v,
            None => panic!("invalid key"),
        }
    }
}

// /// Index value for a slab
// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// #[repr(transparent)]
// pub struct Index(u32);

// impl SlabIndex for Index {
//     const MAX: usize = u32::MAX as usize;

//     fn as_usize(&self) -> usize {
//         self.0 as usize
//     }

//     fn from_usize(index: usize) -> Self
//     where
//         Self: Sized,
//     {
//         Self(index as u32)
//     }
// }

// impl Deref for Index {
//     type Target = u32;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl From<usize> for Index {
//     fn from(val: usize) -> Self {
//         Self(val as u32)
//     }
// }

// impl From<u32> for Index {
//     fn from(val: u32) -> Self {
//         Self(val)
//     }
// }

// impl From<u16> for Index {
//     fn from(val: u16) -> Self {
//         Self(val as u32)
//     }
// }

// impl From<Index> for usize {
//     fn from(idx: Index) -> Self {
//         idx.0 as usize
//     }
// }
