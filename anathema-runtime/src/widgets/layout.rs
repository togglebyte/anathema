//! Storing the calculated layout of each element.
//!
//! The calculating the layout it self is done elsewhere.
use anathema_geometry::{Pos, Region, Size};
use anathema_store::secondary_map::{GenerationalStorage, SecondaryMap};

use crate::elements::ElementId;

/// Store the layout for each element.
#[derive(Debug)]
pub struct Layout {
    regions: SecondaryMap<GenerationalStorage<ElementId, Region>>,
}

impl Layout {
    pub(crate) fn empty() -> Self {
        Self {
            regions: SecondaryMap::empty(),
        }
    }

    pub(crate) fn set_size(&mut self, id: ElementId, size: Size) {
        self.regions[id].resize(size);
    }

    pub(crate) fn set_pos(&mut self, id: ElementId, pos: Pos) {
        self.regions[id].set_pos(pos);
    }

    pub(crate) fn insert(&mut self, id: ElementId) {
        self.regions.insert(id, Region::ZERO);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Region> {
        self.regions.iter().map(|(_, region)| region)
    }
}
