use anathema_store::key;
use anathema_store::slab::{Key, SecondaryMap};

use crate::runtime::elements::ElementId;

key!(ScopeId);

struct ScopeNode {
    //
    parent: Option<ElementId>,
}

pub struct Scope {
    scopes: SecondaryMap<ElementId, ScopeNode>,
}

impl Scope {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }

    pub fn lookup(&self, key: &str, id: ScopeId) -> () {}
}
