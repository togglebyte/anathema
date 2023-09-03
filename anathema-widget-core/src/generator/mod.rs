use std::fmt::Debug;

mod expressions;
mod nodes;
mod values;

// #[cfg(test)]
// mod testing;

use std::rc::Rc;

use anathema_values::{Context, State, Scope};
pub use expressions::{Loop, SingleNode, Expression};
pub use nodes::Nodes;
pub use values::Attributes;

pub fn make_it_so(expressions: Vec<Expression>) -> Nodes {
    Nodes::new(expressions.into(), 0.into())
}
