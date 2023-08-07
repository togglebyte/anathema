// #![deny(missing_docs)]
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::marker::PhantomData;

use anathema_render::Color;

use crate::hashmap::HashMap;
use crate::path::PathId;
pub use crate::values::{List, Map, Container};

/// A value reference.
/// Used an index to lookup values
pub struct ValueRef<T> {
    pub(crate) index: usize,
    pub(crate) gen: usize,
    _p: PhantomData<T>,
}

impl<T> ValueRef<T> {
    pub fn new(index: usize, gen: usize) -> Self {
        Self {
            index,
            gen,
            _p: PhantomData,
        }
    }
}

impl<T> Hash for ValueRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.index, self.gen).hash(state)
    }
}

impl<T> Eq for ValueRef<T> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<T> PartialEq for ValueRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index.eq(&other.index) && self.gen.eq(&other.gen)
    }
}

impl<T> Clone for ValueRef<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            gen: self.gen,
            _p: PhantomData,
        }
    }
}

impl<T> Copy for ValueRef<T> {}

impl<T> Debug for ValueRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueRef")
            .field("index", &self.index)
            .field("gen", &self.gen)
            .finish()
    }
}
