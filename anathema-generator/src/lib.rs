use std::fmt::Debug;

mod expressions;
mod nodes;
mod values;

// #[cfg(test)]
mod testing;

use std::rc::Rc;

use anathema_values::State;
use expressions::Expression;
pub use nodes::{NodeId, Nodes};
pub use values::{Attributes, Value};

pub trait IntoWidget: Sized + Debug {
    type Meta: ?Sized + Debug;
    type State: State;
    type Err;

    fn create_widget(meta: &Rc<Self::Meta>, state: &Self::State, attributes: &Attributes) -> Result<Self, Self::Err>;

    fn layout(&mut self, children: &mut Nodes<Self>);
}

pub fn make_it_so<Widget: IntoWidget>(expressions: Vec<Expression<Widget>>) -> Nodes<Widget> {
    panic!()
    // Nodes::new(Rc::new(expressions), EvalState::new())
}
