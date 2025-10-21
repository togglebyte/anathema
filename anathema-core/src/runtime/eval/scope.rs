use anathema_state::StateId;
use anathema_store::key;
use anathema_store::slab::{Key, SecondaryMap};

use crate::runtime::components::ComponentId;
use crate::runtime::elements::{ElementId, Elements};

#[derive(Debug, Copy, Clone)]
pub(super) enum ScopeKey<'a> {
    Attributes,
    State,
    Key(&'a str),
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Entry<'bp> {
    // TODO: gross, this is supposed to be a component id
    State(ComponentId),

    Attributes(Key),
    Value { key: &'bp str, value: () },
}

impl PartialEq<Entry<'_>> for ScopeKey<'_> {
    fn eq(&self, other: &Entry<'_>) -> bool {
        match (self, other) {
            (ScopeKey::Attributes, Entry::Attributes(_)) => true,
            (ScopeKey::Attributes, _) => false,

            (ScopeKey::State, Entry::State(_)) => true,
            (ScopeKey::State, _) => false,

            (ScopeKey::Key(lhs), Entry::Value { key: rhs, .. }) => lhs == rhs,
            (_, _) => false,
        }
    }
}

struct ScopeNode<'bp> {
    entries: Vec<Entry<'bp>>,
    boundary: bool,
}

impl<'bp> ScopeNode<'bp> {
    fn get(&self, key: ScopeKey<'_>) -> Option<&Entry<'bp>> {
        self.entries.iter().find(|entry| key.eq(entry))
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
                        None if node.boundary => break None,
                        None => id = elements[id].parent?,
                    }
                }
                None => {
                    // and we are NOT on a scope boundary
                    id = elements[id].parent?;
                }
            }
        }
    }

    pub(crate) fn push_component(&mut self, element: ElementId, component: ComponentId) {
        match self.scopes.get_mut(element) {
            Some(node) => node
                .entries
                .extend_from_slice(&[Entry::State(component), Entry::Attributes(element.into())]),
            None => self.scopes.insert(
                element,
                ScopeNode {
                    entries: vec![Entry::State(component), Entry::Attributes(element.into())],
                    boundary: true,
                },
            ),
        }
    }
}
