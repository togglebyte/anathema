use anathema_store::slab::SecondaryMap;

use crate::nodes::NodeId;

pub struct Scope {
    scopes: SecondaryMap<NodeId, ()>,
}

impl Scope {
    pub fn empty() -> Self {
        Self {
            scopes: SecondaryMap::empty(),
        }
    }
}
