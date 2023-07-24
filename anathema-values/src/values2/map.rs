use crate::hashmap::IntMap;
use crate::{ValueRef, ValueV2};

#[derive(PartialEq)]
pub struct Map<T>(IntMap<usize, ValueRef<ValueV2<T>>>);

impl<T> Map<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> From<IntMap<usize, ValueRef<ValueV2<T>>>> for Map<T> {
    fn from(v: IntMap<usize, ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}
