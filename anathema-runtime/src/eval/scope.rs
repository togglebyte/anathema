use anathema_store::slab::{Index, Key, SecondaryMap};

use crate::components::ComponentId;
use crate::elements::{ElementId, Elements};
use crate::eval::values::TemplateValue;
use crate::AnonValue;

// The value key for a scope entry
#[derive(Debug, Copy, Clone)]
pub(super) enum ScopeKey<'a> {
    Attributes,
    State,
    Key(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ScopeId(ElementId);

impl From<ElementId> for ScopeId {
    fn from(value: ElementId) -> Self {
        ScopeId(value)
    }
}

impl From<ScopeId> for Index {
    fn from(value: ScopeId) -> Self {
        value.0.into()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Entry<'bp> {
    State(ComponentId),
    Attributes(ElementId),
    Value {
        key: &'bp str,
        value: TemplateValue<'bp>,
    },
    Iteration {
        key: &'bp str,
        value: TemplateValue<'bp>,
        loop_counter: AnonValue,
    },
}

impl PartialEq<Entry<'_>> for ScopeKey<'_> {
    fn eq(&self, other: &Entry<'_>) -> bool {
        match (self, other) {
            (ScopeKey::Attributes, Entry::Attributes(_)) => true,
            (ScopeKey::Attributes, _) => false,

            (ScopeKey::State, Entry::State(_)) => true,
            (ScopeKey::State, _) => false,

            (ScopeKey::Key(lhs), Entry::Value { key: rhs, .. }) => lhs == rhs,
            (ScopeKey::Key(lhs), Entry::Iteration { key: rhs, .. }) => lhs == rhs,
            (_, _) => false,
        }
    }
}

#[derive(Debug)]
struct ScopeNode<'bp> {
    entries: Vec<Entry<'bp>>,
    boundary: bool,
}

impl<'bp> ScopeNode<'bp> {
    fn get(&self, key: ScopeKey<'_>) -> Option<&Entry<'bp>> {
        self.entries.iter().find(|entry| key.eq(entry))
    }
}

#[derive(Debug)]
pub(crate) struct Scope<'bp> {
    scopes: SecondaryMap<ScopeId, ScopeNode<'bp>>,
}

impl<'bp> Scope<'bp> {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }

    /// Find the closest scoped id from a given element id
    pub fn nearest_scope_id(&self, id: ElementId, elements: &Elements<'_>) -> Option<ScopeId> {
        let mut id = ScopeId(id);
        loop {
            match self.scopes.get(id) {
                Some(node) => break Some(id),
                None => id = ScopeId(elements[id.0].parent?),
            }
        }
    }

    pub fn lookup(&self, key: ScopeKey<'_>, mut id: ScopeId, elements: &Elements<'bp>) -> Option<Entry<'bp>> {
        // Try to get until we reach a scope boundary

        loop {
            match self.scopes.get(id) {
                Some(node) => {
                    // If the scope node contains the key then fetch the value
                    match node.get(key).cloned() {
                        val @ Some(_) => break val,
                        None if node.boundary => break None,
                        None => id = ScopeId(elements[id.0].parent?),
                    }
                }
                None => id = ScopeId(elements[id.0].parent?),
            }
        }
    }

    pub(crate) fn push_component(&mut self, component_element: ElementId, component: ComponentId) {
        let scope_id = ScopeId(component_element);

        match self.scopes.get_mut(scope_id) {
            Some(node) => node
                .entries
                .extend_from_slice(&[Entry::State(component), Entry::Attributes(component_element)]),
            None => self.scopes.insert(
                scope_id,
                ScopeNode {
                    entries: vec![Entry::State(component), Entry::Attributes(component_element)],
                    boundary: true,
                },
            ),
        }
    }

    pub(crate) fn scope_iteration(
        &mut self,
        iter_element: ElementId,
        key: &'bp str,
        value: TemplateValue<'bp>,
        loop_counter: AnonValue,
    ) {
        let scope_id = ScopeId(iter_element);

        let entry = Entry::Iteration {
            key,
            value,
            loop_counter,
        };

        match self.scopes.get_mut(scope_id) {
            Some(node) => node.entries.push(entry),
            None => self.scopes.insert(
                scope_id,
                ScopeNode {
                    entries: vec![entry],
                    boundary: false,
                },
            ),
        }
    }
}
