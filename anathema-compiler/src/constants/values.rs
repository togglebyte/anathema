use anathema_values::Slab;
use anathema_widget_core::Value;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueId(usize);

#[derive(Debug)]
pub struct Values(Slab<Value>);

impl Values {
    pub(crate) fn empty() -> Self {
        Self(Slab::empty())
    }

    pub(crate) fn push(&mut self, value: Value) -> ValueId {
        ValueId(self.0.push(value))
    }

    pub(crate) fn get(&self, index: ValueId) -> Option<&Value> {
        self.0.get(index.0)
    }
}
