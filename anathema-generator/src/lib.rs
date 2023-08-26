mod expressions;
mod nodes;
mod values;

// #[cfg(test)]
mod testing;

use std::rc::Rc;

use anathema_values::State;
use expressions::{EvalState, Expressions};
pub use nodes::{NodeId, Nodes};
pub use values::{Attributes, Value};

// TODO: rename this amazingly named trait
pub trait Flap: Sized + std::fmt::Debug {
    type Meta: ?Sized + std::fmt::Debug;
    type Err;

    fn do_it(meta: &Rc<Self::Meta>, state: &impl State) -> Result<Self, Self::Err>;
}

pub fn make_it_so<Ctx: Flap>(expressions: Expressions<Ctx>) -> Nodes<Ctx> {
    Nodes::new(Rc::new(expressions), EvalState::new())
}
