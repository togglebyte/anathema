pub use query::Queryable;

use crate::components::{ComponentId, Components};
use crate::nodes::Nodes;

mod component;
mod components;
mod constraints;
mod elements;
mod nodes;
mod query;

pub struct UI<'a> {
    nodes: Nodes,
    components: Components,
    meh: &'a (),
}

impl<'a> UI<'a> {
    /// Dispatch events
    fn dispatch(&mut self, component: ComponentId) {}

    fn this(&self) -> () {}

    // This should really be by_tag, by_name, by_id etc.
    fn query(&self) -> () {}
}

impl<'a> Queryable for &mut UI<'a> {
    fn by_tag(self) -> impl Queryable {
        self
    }
}
