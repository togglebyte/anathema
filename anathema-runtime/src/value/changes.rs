use crate::elements::ElementId;
use crate::eval::scope::ScopeId;
use anathema_compiler::expressions::ExpressionId;
use anathema_store::stack::Stack;

pub type Changes = Stack<(ValueIndex, Change)>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Change {
    /// A value was inserted into a list
    Inserted(u32),
    /// A value was removed from a list
    Removed(u32),
    /// A value has changed
    Changed,
    /// Value was removed (e.g removed from a map)
    Dropped,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueIndex(ExpressionId, Option<ScopeId>);

impl ValueIndex {
    pub fn new(expression: ExpressionId, scope: Option<ScopeId>) -> Self {
        Self(expression, scope)
    }

    pub(crate) fn consume(self) -> (ExpressionId, Option<ScopeId>) {
        (self.0, self.1)
    }
}
