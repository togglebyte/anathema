use std::cell::RefMut;

use anathema_geometry::Size;

use crate::elements::{Element, ElementId, Elements};
use crate::widgets::{Layouts, Widget};

// TODO this can probably be removed

pub struct ElementChildren<'a, 'bp> {
    children: &'a [ElementId],
    elements: &'a Elements<'bp>,
    index: usize,
}

impl<'a, 'bp> Iterator for ElementChildren<'a, 'bp> {
    type Item = &'a Element<'bp>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.children.len() {
            return None;
        }

        let id = self.children[self.index];
        self.index += 1;

        let node = &self.elements[id];

        Some(&node.element)
    }
}
