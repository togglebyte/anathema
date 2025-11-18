use std::ops::Range;

use anathema_geometry::Region;

use crate::crossterm::Cell;

pub(super) struct Buffer {
    inner: Vec<Cell>,
}

impl Buffer {
    pub(super) fn new(width: usize, height: usize) -> Self {
        Self {
            inner: vec![Cell::default(); width * height],
        }
    }

    pub(super) fn copy_range(&mut self, range: Range<usize>, src: &Self) {
        self.inner[range.start..range.end].clone_from_slice(&src.inner[range]);
    }

    pub(crate) fn cells(&self, range: Range<usize>) -> &[Cell] {
        &self.inner[range]
    }

    pub(crate) fn slice_mut(&mut self, range: Range<usize>) -> &mut [Cell] {
        &mut self.inner[range]
    }

    pub(crate) fn write(&mut self, index: usize, state: super::State) {
        self.inner[index].state = state;
    }
}
