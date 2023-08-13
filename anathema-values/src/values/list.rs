use std::fmt::{self, Debug};
use std::ops::Index;

use crate::ValueRef;

#[derive(PartialEq)]
pub struct List<T>(Vec<ValueRef<T>>);

impl<T> List<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[ValueRef<T>] {
        self.0.as_slice()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValueRef<T>> + '_ {
        self.0.iter()
    }
}

impl<T> From<Vec<ValueRef<T>>> for List<T> {
    fn from(v: Vec<ValueRef<T>>) -> Self {
        Self(v)
    }
}

impl<T> Index<usize> for List<T> {
    type Output = ValueRef<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("List")
            .field(&self.0)
            .finish()
    }
}
