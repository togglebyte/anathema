use std::fmt::Debug;

mod expressions;
mod nodes;
mod values;

#[cfg(any(attribute = "testing", test))]
mod testing;

use std::rc::Rc;

use anathema_values::{Context, State, Scope};
pub use expressions::{Loop, SingleNode, Expression, If, Else, ControlFlow};
pub use nodes::Nodes;

pub fn make_it_so(expressions: &[Expression]) -> Nodes<'_> {
    Nodes::new(expressions, 0.into())
}
