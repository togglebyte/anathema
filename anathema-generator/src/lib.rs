use std::fmt::Debug;

mod expressions;
mod nodes;
mod values;

#[cfg(test)]
mod testing;

use std::rc::Rc;

use anathema_values::{Context, State, Scope};
pub use expressions::{Loop, SingleNode, Expression};
pub use nodes::{NodeId, Nodes};
pub use values::Attributes;

pub trait IntoWidget: Sized + Debug {
    type Meta: ?Sized + Debug;
    type Err;
    type Widget: Debug;
    type Output;

    fn create_widget(&self, context: Context<'_, '_>, attributes: &Attributes) -> Result<Self::Widget, Self::Err>;
}

pub fn make_it_so<Widget: IntoWidget>(expressions: Vec<Expression<Widget>>) -> Nodes<Widget> {
    Nodes::new(expressions.into(), 0.into())
}
