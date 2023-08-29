use anathema_values::{Slab, ScopeValue};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueId(usize);

#[derive(Debug)]
pub struct Values(Slab<ScopeValue>);

impl Values {
    pub(crate) fn empty() -> Self {
        Self(Slab::empty())
    }

    pub(crate) fn push(&mut self, value: ScopeValue) -> ValueId {
        ValueId(self.0.push(value))
    }

    pub(crate) fn get(&self, index: ValueId) -> Option<&ScopeValue> {
        self.0.get(index.0)
    }
}
