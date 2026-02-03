//! Storing the calculated layout of each element.
//!
//! The calculating the layout it self is done elsewhere.
use std::ops::Index;

use anathema_geometry::{Pos, Region, Size};
use anathema_store::secondary_map::{GenerationalStorage, SecondaryMap};

use crate::elements::ElementId;

/// A widget layout size containing both the inner
/// and the outer size.
#[derive(Debug, Copy, Clone)]
pub struct LayoutSize {
    pub inner: Size,
    pub outer: Size,
}

impl LayoutSize {
    const ZERO: Self = Self {
        inner: Size::ZERO,
        outer: Size::ZERO,
    };

    pub fn new(inner: Size, outer: Size) -> Self {
        Self { inner, outer }
    }

    pub fn same(size: Size) -> Self {
        Self {
            inner: size,
            outer: size,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pos: Pos,
    size: LayoutSize,
}

impl Layout {
    const ZERO: Self = Layout {
        pos: Pos::ZERO,
        size: LayoutSize::ZERO,
    };

    fn new(size: LayoutSize) -> Self {
        Self {
            pos: Pos::ZERO,
            size,
        }
    }

    pub fn region(self) -> Region {
        Region::from((self.pos, self.size.outer))
    }

    fn resize(&mut self, size: LayoutSize) {
        self.size = size;
    }
}

/// Store the layout for each element.
#[derive(Debug)]
pub struct Layouts {
    inner: SecondaryMap<GenerationalStorage<ElementId, Layout>>,
}

impl Layouts {
    pub(crate) fn empty() -> Self {
        Self {
            inner: SecondaryMap::empty(),
        }
    }

    pub(crate) fn set_size(&mut self, id: ElementId, size: LayoutSize) {
        match self.inner.get_mut(id) {
            Some(layout) => layout.resize(size),
            None => self.inner.insert(id, Layout::new(size)),
        }
    }

    pub(crate) fn set_pos(&mut self, id: ElementId, pos: Pos) {
        self.inner[id].pos = pos;
    }

    pub(crate) fn insert(&mut self, id: ElementId) {
        self.inner.insert(id, Layout::ZERO);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Region> {
        self.inner.iter().map(|(_, layout)| layout.region())
    }
}

impl Index<ElementId> for Layouts {
    type Output = Layout;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.inner[index]
    }
}
