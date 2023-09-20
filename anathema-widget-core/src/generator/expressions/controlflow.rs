use anathema_values::{ScopeValue, Scope, State, ValueExpr, NodeId};

use super::Expression;

#[derive(Debug)]
pub struct If {
    pub cond: ValueExpr,
    pub body: Vec<Expression>,
}

// TODO: need this to work
// y = false
// z = true
// for x in [0, 1, 2]
//     if (x || y) || z
//         text "it was true"

impl If {
    pub(super) fn is_true(&self, scope: &Scope<'_>, state: &mut dyn State, node_id: Option<&NodeId>) -> bool {
        panic!()
    }
}

#[derive(Debug)]
pub struct Else {
    pub cond: Option<ValueExpr>,
    pub body: Vec<Expression>,
}
