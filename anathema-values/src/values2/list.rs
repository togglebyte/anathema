use std::fmt::{self, Debug};
use std::ops::Index;

use crate::{ValueRef, ValueV2};

#[derive(PartialEq)]
pub struct List<T>(Vec<ValueRef<ValueV2<T>>>);

impl<T> List<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[ValueRef<ValueV2<T>>] {
        self.0.as_slice()
    }
}

impl<T> From<Vec<ValueRef<ValueV2<T>>>> for List<T> {
    fn from(v: Vec<ValueRef<ValueV2<T>>>) -> Self {
        Self(v)
    }
}

impl<T> Index<usize> for List<T> {
    type Output = ValueRef<ValueV2<T>>;

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
