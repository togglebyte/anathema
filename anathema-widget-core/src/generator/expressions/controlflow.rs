use anathema_values::{ScopeValue, Scope, State};

use super::Expression;

#[derive(Debug)]
pub struct If {
    pub cond: ScopeValue,
    pub body: Vec<Expression>,
}

// TODO: need this to work
// y = false
// z = true
// for x in [0, 1, 2]
//     if (x || y) || z
//         text "it was true"

impl If {
    pub(super) fn is_true<S: State>(&self, scope: &Scope<'_>, state: &mut S) -> bool {
        panic!()
    }
}

#[derive(Debug)]
pub struct Else {
    pub cond: Option<ScopeValue>,
    pub body: Vec<Expression>,
}
