use std::cell::RefMut;

use anathema_geometry::Size;

use crate::elements::{Element, ElementId, Elements};
use crate::widgets::{Layout, Widget};

/// Children of a given widget.
pub struct Children<'a, 'bp> {
    children: &'a [ElementId],
    elements: &'a Elements<'bp>,
    index: usize,
}

impl<'a, 'bp> Iterator for Children<'a, 'bp> {
    type Item = WidgetRef<'a, 'bp>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.children.len() {
            return None;
        }

        let id = self.children[self.index];
        self.index += 1;

        let node = &self.elements[id];
        let Element::Widget(widget) = &node.element else { unreachable!() };

        let children = Children {
            children: &node.children,
            elements: self.elements,
            index: 0,
        };

        let widget_ref = WidgetRef {
            id,
            widget: widget.borrow_mut(),
            children,
        };

        Some(widget_ref)
    }
}

pub struct WidgetRef<'a, 'bp> {
    id: ElementId,
    widget: RefMut<'a, Box<dyn Widget>>,
    children: Children<'a, 'bp>,
}

impl<'a, 'bp> WidgetRef<'a, 'bp> {
    pub fn layout(mut self, layout: &mut Layout) -> Size {
        self.widget.layout(self.children, layout)
    }
}
