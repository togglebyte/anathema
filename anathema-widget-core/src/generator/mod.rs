mod expressions;
mod nodes;
mod values;

#[cfg(any(attribute = "testing", test))]
mod testing;

pub use expressions::{ControlFlow, Else, Expression, If, Loop, SingleNode};
pub use nodes::Nodes;

pub fn make_it_so(expressions: &[Expression]) -> Nodes<'_> {
    Nodes::new(expressions, 0.into())
}
