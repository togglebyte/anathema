use std::borrow::Cow;

use anathema_state::{PendingValue, StateId};
use anathema_store::slab::Key;

use crate::expression::{Kind, ValueExpr};
use crate::{Collection, ValueKind};

pub enum Lookup {
    State(StateId),
    ComponentProperties(Key),
}

#[derive(Debug)]
pub(crate) enum Entry<'parent, 'bp> {
    Component { state: StateId, component_attributes: Key },
    Value(&'parent ValueExpr<'bp>),
    Collection(&'bp str, &'parent Collection<'bp>),
    Index(&'bp str, usize),
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

    pub fn with_collection(
        binding: &'bp str,
        collection: &'parent Collection<'bp>,
        parent: &'parent Scope<'parent, 'bp>,
    ) -> Self {
        let value = Entry::Collection(binding, collection);
        Self {
            parent: Some(parent),
            value,
        }
    }

    pub fn with_index(binding: &'bp str, index: usize, parent: &'parent Scope<'parent, 'bp>) -> Self {
        let value = Entry::Index(binding, index);
        Self {
            value,
            parent: Some(parent),
        }
    }

    pub fn root() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self::new(Entry::Empty)
    }

    pub(crate) fn get_state(&self) -> Option<StateId> {
        match &self.value {
            Entry::Component { state, .. } => Some(*state),
            _ => self.parent?.get_state(),
        }
    }

    pub(crate) fn get_attributes(&self) -> Option<Key> {
        match &self.value {
            Entry::Component {
                component_attributes, ..
            } => Some(*component_attributes),
            _ => self.parent?.get_attributes(),
        }
    }

    pub(crate) fn lookup(&self, key: &str) -> Option<ValueExpr<'bp>> {
        match self.value {
            Entry::Index(binding, index) if key == binding => {
                match self.parent.expect("the parent can only be a collection").value {
                    Entry::Collection(_, collection) => {
                        match collection.0.kind {
                            ValueKind::List(_) => {
                                // Since this is a static list we can just clone the value
                                // expression here and return that, since it's already evaluated
                                match &collection.0.expr {
                                    ValueExpr::List(list) => {
                                        let value = list[index].clone();
                                        Some(value)
                                    }
                                    _ => unreachable!("the expression can only be a list"),
                                }
                            }
                            ValueKind::DynList(value) => {
                                let state = value.as_state()?;
                                let list = state.as_any_list()?;
                                let value = list.lookup(index)?;
                                Some(value.into())
                            }
                            _ => unreachable!("none of the other values can be a collection"),
                        }
                    }
                    _ => unreachable!("the parent scope is always a collection"),
                }
            }
            _ => self.parent?.lookup(key),
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
