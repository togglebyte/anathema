use crate::{ValueRef, ValueV2};

#[derive(PartialEq)]
pub struct List<T>(Vec<ValueRef<ValueV2<T>>>);

impl<T> List<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> From<Vec<ValueRef<ValueV2<T>>>> for List<T> {
    fn from(v: Vec<ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}
