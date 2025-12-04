// #![deny(missing_docs)]
//! A slab reduces the number of allocations and keeps fixed indices, unlike a vector.
//! Inserting a value into a slab returns the index of the value.
//! Removing a value from the slab does not shift the subsequent values down like it would
//! in a vector, but reserves the position of the previous value as a vacant value for the next
//! insertion.
//!
//! A slab has two immediate advantages:
//! * Reduce allocations
//! * Fixed indices

pub use self::basic::Basic;
pub use self::generational::{Gen, Generational, Key};
pub use self::shared::arc::{ArcElement, ArcSlab};
pub use self::shared::rc::{RcElement, RcSlab};
// pub use self::shared::Shared;
pub use self::sparse::Sparse;

mod basic;
mod generational;
mod shared;
mod sparse;

pub trait Slab: Default {
    type Key: Copy + PartialEq;
    type Value;

    /// Insert a value into the slab, returning the index
    fn insert(&mut self, value: Self::Value) -> Self::Key;

    // /// Insert a value at a given index.
    // /// This will force the underlying storage to grow if
    // /// the index given is larger than the current capacity.
    // ///
    // /// This will overwrite any value currently at that index.
    // ///
    // /// # Panics
    // ///
    // /// Panics if a value is inserted at a position that is currently checked out
    // fn insert_at(&mut self, key: Self::Key, value: Self::Value);

    /// Removes a value out of the slab.
    /// This assumes the value exists
    ///
    /// # Panics
    /// Will panic if the slot is not occupied
    fn remove(&mut self, key: Self::Key) -> Self::Value;

    fn remove_if<F>(&mut self, key: Self::Key, pred: F) -> Option<Self::Value>
    where
        F: Fn(&Self::Value) -> bool,
    {
        let remove = self.get(key).map(|val| pred(val)).unwrap_or(false);
        if remove {
            return Some(self.remove(key));
        }

        None
    }

    /// Get a reference to a value
    fn get(&self, key: Self::Key) -> Option<&Self::Value>;

    /// Get a mutable reference to a value
    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;

    /// Returns true if a given key exists
    fn contains_key(&mut self, key: Self::Key) -> bool;

    /// # Performance notes
    ///
    /// Be aware that this will only ever be as performant as
    /// the underlying vector if all entries are occupied.
    ///
    /// E.g if the only occupied slot is at index 1,000,000, then this will
    /// iterate over 999,999 entries to get there.
    fn iter(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)>;

    /// # Performance notes
    ///
    /// See [`Self::iter`]
    fn iter_mut(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)>;

    /// Return an iterator of keys
    fn iter_keys(&self) -> impl Iterator<Item = Self::Key>;

    /// Return an iterator over value references
    fn iter_values(&self) -> impl Iterator<Item = &Self::Value>;

    /// Return an iterator over mutable references
    fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut Self::Value>;
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
