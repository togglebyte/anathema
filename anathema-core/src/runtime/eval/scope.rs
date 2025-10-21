use anathema_state::StateId;
use anathema_store::key;
use anathema_store::slab::{Key, SecondaryMap};

use crate::runtime::elements::{ElementId, Elements};

#[derive(Debug, Copy, Clone)]
pub(super) enum ScopeKey<'a> {
    Attributes,
    State,
    Key(&'a str),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Entry<'bp> {
    State(StateId),
    Attributes(Key),
    Value { key: &'bp str, value: () },
}

impl PartialEq<Entry<'_>> for ScopeKey<'_> {
    fn eq(&self, other: &Entry<'_>) -> bool {
        todo!()
    }
}

struct ScopeNode<'bp>(Vec<Entry<'bp>>);

impl<'bp> ScopeNode<'bp> {
    fn get(&self, key: ScopeKey<'_>) -> Option<&Entry<'bp>> {
        self.0.iter().find(|entry| key.eq(entry))
    }
}

pub(crate) struct Scope<'bp> {
    scopes: SecondaryMap<ElementId, ScopeNode<'bp>>,
}

impl<'bp> Scope<'bp> {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }

    pub fn lookup(&self, key: ScopeKey<'_>, mut id: ElementId, elements: &Elements<'bp>) -> Option<Entry<'bp>> {
        // Try to get until we reach a scope boundary

        // panic!("if the value is a state but its not the key then that's the boundary");

        loop {
            match self.scopes.get(id) {
                Some(node) => {
                    // If the scope node contains the key then fetch the value
                    match node.get(key).copied() {
                        val @ Some(_) => break val,
                        None => {
                            //     * and is NOT a boundary, then look in the parent
                            //     * and IS a boundary, then return None
                        }
                    }
                }
                None => {
                    // and we are NOT on a scope boundary
                    id = elements[id].parent?;
                }
            }
        }
    }

    pub(crate) fn push_component(&mut self, element: ElementId, state: StateId) {
        match self.scopes.get_mut(element) {
            Some(node) => node
                .0
                .extend_from_slice(&[Entry::State(state), Entry::Attributes(element.into())]),
            None => self.scopes.insert(
                element,
                ScopeNode(vec![Entry::State(state), Entry::Attributes(element.into())]),
            ),
        }
    }
}
