// #![deny(missing_docs)]
use std::fmt::{self, Debug};
use std::marker::PhantomData;

use anathema_render::Color;

use crate::hashmap::HashMap;
use crate::path::PathId;
pub use crate::values2::{List, Map, ValueV2};

/// A value reference.
/// Used an index to lookup values
#[derive(PartialEq)]
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
