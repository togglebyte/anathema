use anathema_state::StateId;
use anathema_store::key;
use anathema_store::slab::{Key, SecondaryMap};

use crate::runtime::elements::{ElementId, Elements};

#[derive(Debug, Copy, Clone)]
enum Entry {
    State(StateId),
}

enum ScopeNode<'bp> {
    One(&'bp str, Entry),
    Many(Vec<(&'bp str, Entry)>),
}

impl<'bp> ScopeNode<'bp> {
    fn get(&self, key: &str) -> Option<&Entry> {
        match self {
            ScopeNode::One(k, entry) if key == *k => Some(entry),
            ScopeNode::One(..) => None,
            ScopeNode::Many(values) => values.iter().find(|(k, _)| key == *k).map(|(_, val)| val)
        }
    }
}

pub struct Scope<'bp> {
    scopes: SecondaryMap<ElementId, ScopeNode<'bp>>,
}

impl<'bp> Scope<'bp> {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }

    pub fn lookup(&self, key: &str, mut id: ElementId, elements: &Elements<'bp>) -> Option<&Entry> {
        // Try to get until we reach a scope boundary

        panic!("if the value is a state but its not the key then that's the boundary");

        loop {
            match self.scopes[id].get(key) {
                val @ Some(_) => break val,
                None => {
                    // and we are NOT on a scope boundary
                    id = elements[id].parent?;
                }
            }
        }
    }

    pub(crate) fn push_state(&mut self, element: ElementId, state_id: StateId) {
        match self.scopes.get_mut(element) {
            Some(ScopeNode::One(k, v)) => todo!(),
            Some(ScopeNode::Many(entries)) => entries.push(("state", Entry::State(state_id))),
            None => self.scopes.insert(element, ScopeNode::One("state", Entry::State(state_id))),
        }
    }
}
