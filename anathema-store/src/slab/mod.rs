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

pub use self::basic::Slab;
pub use self::generational::{Gen, GenSlab, Key};
pub use self::rc::{Element, RcSlab};
pub use self::secondary_map::SecondaryMap;

mod basic;
mod generational;
mod rc;
mod secondary_map;

/// Index value for a slab
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Index(u32);

impl Deref for Index {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Index {
    fn from(val: usize) -> Self {
        Self(val as u32)
    }
}

impl From<u32> for Index {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<Index> for usize {
    fn from(idx: Index) -> Self {
        idx.0 as usize
    }
}

/// A ticket used when checkout an entry out of the slab.
#[derive(Debug)]
pub struct Ticket<I, T> {
    pub(crate) value: T,
    key: I,
}

impl<I, T> Deref for Ticket<I, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<I, T> DerefMut for Ticket<I, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
