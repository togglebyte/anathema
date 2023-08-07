use anathema_values::Slab;
use anathema_widget_core::Value;
use anathema_generator::ExpressionAttribute;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueId(usize);

pub struct Values(Slab<ExpressionAttribute<Value>>);

impl Values {
    pub(crate) fn empty() -> Self {
        Self(Slab::empty())
    }

    pub(crate) fn push(&mut self, value: ExpressionAttribute<Value>) -> ValueId {
        ValueId(self.0.push(value))
    }

    pub(crate) fn get(&self, index: ValueId) -> Option<&ExpressionAttribute<Value>> {
        self.0.get(index.0)
    }
}
