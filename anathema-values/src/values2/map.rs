use std::fmt::{self, Debug};
use crate::hashmap::IntMap;
use crate::{ValueRef, ValueV2};

#[derive(PartialEq)]
pub struct Map<T>(IntMap<ValueRef<ValueV2<T>>>);

impl<T> Map<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> () {
        self.0.iter();
    }
}


impl<T> From<IntMap<ValueRef<ValueV2<T>>>> for Map<T> {
    fn from(v: IntMap<ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}

impl<T> Debug for Map<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Map")
            .field(&self.0)
            .finish()
    }
}

impl<T> Clone for Map<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

