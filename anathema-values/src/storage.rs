use std::ops::Index;

use crate::ValueExpr;
use crate::Slab;

#[derive(Debug)]
pub struct Storage<T>(pub(crate) Slab<T>);

impl<T> Storage<T> {
    pub fn empty() -> Self {
        Self(Slab::empty())
    }

    // De-duplicate values.
    // If the value already exist, just return the value position,
    pub fn push(&mut self, value: T) -> usize
    where
        T: PartialEq,
    {
        self.0.find(&value).unwrap_or_else(|| self.0.push(value))
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueId(usize);

impl From<usize> for ValueId {
    fn from(n: usize) -> Self {
        Self(n)
    }
}

#[derive(Debug)]
pub struct Values(Storage<ValueExpr>);

impl Values {
    pub fn empty() -> Self {
        Self(Storage::empty())
    }

    pub fn push(&mut self, value: ValueExpr) -> ValueId {
        ValueId(self.0.push(value))
    }

    pub fn get(&self, index: ValueId) -> Option<&ValueExpr> {
        self.0.get(index.0)
    }
}

impl Index<ValueId> for Values {
    type Output = ValueExpr;

    fn index(&self, index: ValueId) -> &Self::Output {
        self.get(index).expect("index should not be out of bounds")
    }
}
