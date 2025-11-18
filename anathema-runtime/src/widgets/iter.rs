use std::cell::RefMut;

use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Size};

use crate::attributes::{AttributeRegistry, WidgetAttributes};
use crate::elements::{Element, ElementId, Elements};
use crate::widgets::{Layout, Widget};

/// Children of a given widget.
pub struct Children<'a, 'bp> {
    children: &'a [ElementId],
    elements: &'a Elements<'bp>,
    attribute_reg: &'a AttributeRegistry<'bp>,
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
            attribute_reg: self.attribute_reg,
            index: 0,
        };

        let attributes = &self.attribute_reg[id];
        let attributes = WidgetAttributes::new(self.elements, node.parent, attributes, self.attribute_reg);

        let widget_ref = WidgetRef {
            id,
            widget: widget.borrow_mut(),
            attributes,
            children,
        };

        Some(widget_ref)
    }
}

pub struct WidgetRef<'a, 'bp> {
    id: ElementId,
    widget: RefMut<'a, Box<dyn Widget<'bp>>>,
    attributes: WidgetAttributes<'a, 'bp>,
    children: Children<'a, 'bp>,
}

impl<'a, 'bp> WidgetRef<'a, 'bp> {
    pub fn layout(mut self, layout: &mut Layout) -> Size {
        self.widget.layout(self.children, self.attributes, layout)
    }

    pub fn position(mut self, pos: Pos) {
        self.widget.position(self.children, self.attributes, pos)
    }

    pub fn paint(mut self, frontend: &mut dyn Frontend) {
        self.widget.paint(self.children, self.attributes, frontend)
    }
}
