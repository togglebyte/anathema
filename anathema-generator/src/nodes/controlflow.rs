use std::rc::Rc;

use anathema_values::ScopeValue;

use crate::expressions::Expression;
use crate::{IntoWidget, Nodes};

#[derive(Debug)]
pub struct If<Widget: IntoWidget> {
    pub cond: ScopeValue,
    pub body: Rc<[Expression<Widget>]>,
}

#[derive(Debug)]
pub struct Else<Widget: IntoWidget> {
    pub cond: Option<ScopeValue>,
    pub body: Rc<[Expression<Widget>]>,
}
