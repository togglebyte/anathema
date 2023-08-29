use anathema_values::{ScopeValue, Scope, State};

use super::Expression;
use crate::IntoWidget;

#[derive(Debug)]
pub struct If<Widget: IntoWidget> {
    pub cond: ScopeValue,
    pub body: Vec<Expression<Widget>>,
}

// TODO: need this to work
// y = false
// z = true
// for x in [0, 1, 2]
//     if (x || y) || z
//         text "it was true"

impl<Widget: IntoWidget> If<Widget> {
    pub(super) fn is_true<S: State>(&self, scope: &Scope<'_>, state: &mut S) -> bool {
        // match cond {
        //     ScopeValue::Static(_) => {}
        // }
        false
    }
}

#[derive(Debug)]
pub struct Else<Widget: IntoWidget> {
    pub cond: Option<ScopeValue>,
    pub body: Vec<Expression<Widget>>,
}
