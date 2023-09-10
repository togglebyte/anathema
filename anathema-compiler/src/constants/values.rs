use anathema_values::ScopeValue;

use super::Storage;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueId(usize);

impl From<usize> for ValueId {
    fn from(n: usize) -> Self {
        Self(n)
    }
}

#[derive(Debug)]
pub struct Values(Storage<ScopeValue>);

impl Values {
    pub(crate) fn empty() -> Self {
        Self(Storage::empty())
    }

    pub(crate) fn push(&mut self, value: ScopeValue) -> ValueId {
        ValueId(self.0.push(value))
    }

    pub(crate) fn get(&self, index: ValueId) -> Option<&ScopeValue> {
        self.0.get(index.0)
    }
}
