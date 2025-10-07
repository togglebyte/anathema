use anathema_store::slab::SecondaryMap;
use anathema_store::smallmap::{SmallIndex, SmallMap};

use crate::nodes::NodeId;

#[derive(Debug)]
pub struct AllAttributes {
    attributes: SecondaryMap<NodeId, Attributes>,
}
impl AllAttributes {
    pub(crate) fn empty() -> Self {
        Self {
            attributes: SecondaryMap::empty(),
        }
    }

    pub(crate) fn insert(&mut self, id: NodeId) {
        self.attributes.insert(id, Attributes::empty());
    }
}

#[derive(Debug)]
pub struct Attributes {
    inner: SmallMap<SmallIndex, ()>,
}

impl Attributes {
    pub(crate) fn empty() -> Self {
        Self {
            inner: SmallMap::empty(),
        }
    }
}
