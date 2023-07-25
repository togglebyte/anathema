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
}

impl<T> From<IntMap<ValueRef<ValueV2<T>>>> for Map<T> {
    fn from(v: IntMap<ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}
