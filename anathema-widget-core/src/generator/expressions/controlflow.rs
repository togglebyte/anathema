use anathema_values::{NodeId, Scope, State, ValueExpr};

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
    pub(super) fn is_true(
        &self,
        _scope: &Scope<'_, '_>,
        _state: &dyn State,
        _node_id: Option<&NodeId>,
    ) -> bool {
        panic!()
    }
}

#[derive(Debug)]
pub struct Else {
    pub cond: Option<ValueExpr>,
    pub body: Vec<Expression>,
}
