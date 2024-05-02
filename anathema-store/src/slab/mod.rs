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
pub use self::generational::{GenSlab, Key};
pub use self::rc::{Element, RcSlab};
pub use self::secondary_map::SecondaryMap;

mod basic;
mod generational;
mod rc;
mod secondary_map;

/// A ticket used when checkout an entry out of the slab.
#[derive(Debug)]
pub struct Ticket<T> {
    pub(crate) value: T,
    key: Key,
}

impl<T> Deref for Ticket<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Ticket<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
