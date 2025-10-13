use anathema_store::slab::{Key, SecondaryMap};

use crate::runtime::elements::ElementId;
use crate::runtime::statements::StatementId;

type ScopeId = Key;

struct ScopeNode {
    //
    parent: Option<ScopeId>,
}

pub struct Scope {
    scopes: SecondaryMap<StatementId, ScopeNode>,
}

impl Scope {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }

    pub fn lookup(&self, key: &str, id: ScopeId) -> () {}
}
