use anathema_geometry::{Pos, Region, Size};
use anathema_store::slab::SecondaryMap;

use crate::nodes::NodeId;

#[derive(Debug)]
pub struct Layout {
    regions: SecondaryMap<NodeId, Region>,
}

impl Layout {
    pub fn empty() -> Self {
        Self {
            regions: SecondaryMap::empty(),        
        }
    }

    pub(crate) fn set_size(&mut self, node_id: NodeId, size: Size) {
        self.regions[node_id].resize(size);
    }

    pub(crate) fn set_pos(&mut self, node_id: NodeId, pos: Pos) {
        self.regions[node_id].set_pos(pos);
    }

    pub(crate) fn insert(&mut self, id: NodeId) {
        self.regions.insert(id, Region::ZERO);
    }
}

