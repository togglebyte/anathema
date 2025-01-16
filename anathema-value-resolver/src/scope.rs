use anathema_state::StateId;
use anathema_store::slab::Key;

use crate::Collection;
use crate::expression::ValueExpr;

pub enum Lookup {
    State(StateId),
    ComponentProperties(Key),
}

#[derive(Debug)]
enum Entry<'parent, 'bp> {
    Component { state: StateId, component_attributes: Key },
    Value(&'parent ValueExpr<'bp>),
    Collection(&'bp str, &'parent Collection<'bp>),
    Empty,
}

#[derive(Debug)]
pub struct Scope<'parent, 'bp> {
    parent: Option<&'parent Scope<'parent, 'bp>>,
    value: Entry<'parent, 'bp>,
}

impl<'parent, 'bp> Scope<'parent, 'bp> {
    pub fn new(value: Entry<'parent, 'bp>) -> Self {
        Self { parent: None, value }
    }

    pub fn with_component(state: StateId, attributes: Key, parent: &'parent Scope<'parent, 'bp>) -> Self {
        Self {
            parent: Some(parent),
            value: Entry::Component {
                state,
                component_attributes: attributes,
            },
        }
    }

    pub fn with_collection(binding: &'bp str, collection: &'parent Collection<'bp>, parent: &'parent Scope<'parent, 'bp>) -> Self {
        let value = Entry::Collection(binding, collection);
        Self { parent: Some(parent), value }
    }

    pub fn with_index(binding: &'bp str, index: usize, parent: &'parent Scope<'parent, 'bp>) -> Self {
        match parent.value {
            Entry::Collection(_, _) => todo!(),
            _ => unreachable!("the parent scope is always a collection")
        }
        panic!()
    }

    pub fn root() -> Self {
        Self::new(Entry::Empty)
    }

    // NOTE: use `new`
    // pub fn insert_state(&mut self, state_id: StateId) {
    //     let entry = Entry::State(state_id);
    //     self.insert_entry(entry);
    // }

    pub(crate) fn get_state(&self) -> Option<StateId> {
        match &self.value {
            Entry::Component { state, .. } => Some(*state),
            _ => self.parent?.get_state(),
        }
    }

    pub(crate) fn get_attributes(&self) -> Option<Key> {
        match &self.value {
            Entry::Component { component_attributes, .. } => Some(*component_attributes),
            _ => self.parent?.get_attributes(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_one() {
        // let mut scope = Scope::new();
        // panic!();
        // scope.scope("key",
    }
}
