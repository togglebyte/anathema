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

    fn create_widget<S: State>(meta: &Rc<Self::Meta>, context: Context<'_, '_, S>, attributes: &Attributes) -> Result<Self, Self::Err>;

    fn layout(&mut self, children: &mut Nodes<Self>);
}

pub fn make_it_so<Widget: IntoWidget>(expressions: Vec<Expression<Widget>>) -> Nodes<Widget> {
    Nodes::new(expressions.into(), 0.into())
}
