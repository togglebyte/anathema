use anathema_state::{Change, SubKey};

use crate::runtime::elements::ElementId;
use crate::runtime::eval::scope::ScopeId;
use crate::templates::ExpressionId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValueIndex(ExpressionId, Option<ScopeId>);

impl ValueIndex {
    pub(crate) fn consume(self) -> (ExpressionId, Option<ScopeId>) {
        (self.0, self.1)
    }
}

impl SubKey for ValueIndex {
    fn changed_many(change: Change, keys: impl Iterator<Item = Self>) {
        todo!()
    }
}
