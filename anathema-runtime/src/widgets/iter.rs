use std::cell::RefMut;
use std::any::Any;

use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Size};

use crate::attributes::{AttributeRegistry, WidgetAttributes};
use crate::constraints::Constraints;
use crate::elements::{Element, ElementId, Elements};
use crate::widgets::{Layouts, Widget};

/// Children of a given widget.
pub struct Children<'a, 'bp> {
    children: &'a [ElementId],
    elements: &'a Elements<'bp>,
    attribute_reg: &'a AttributeRegistry<'bp>,
}

impl<'a, 'bp> Children<'a, 'bp> {
    pub(crate) fn new(
        children: &'a [ElementId],
        elements: &'a Elements<'bp>,
        attribute_reg: &'a AttributeRegistry<'bp>,
    ) -> Self {
        Self {
            children,
            elements,
            attribute_reg,
        }
    }

    pub fn iter<'b>(&'b self) -> ChildrenIter<'a, 'b, 'bp> {
        ChildrenIter {
            children: self,
            index: 0,
        }
    }

    pub fn first(&self) -> Option<WidgetRef<'a, 'bp>> {
        self.iter().next()
    }
}

impl<'a, 'b, 'bp> IntoIterator for &'b Children<'a, 'bp> {
    type Item = WidgetRef<'a, 'bp>;

    type IntoIter = ChildrenIter<'a, 'b, 'bp>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Children of a given widget.
pub struct ChildrenIter<'a, 'b, 'bp> {
    children: &'b Children<'a, 'bp>,
    index: usize,
}


impl<'a, 'b, 'bp> Iterator for ChildrenIter<'a, 'b, 'bp> {
    type Item = WidgetRef<'a, 'bp>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.children.children.len() {
            return None;
        }

        let id = self.children.children[self.index];
        self.index += 1;

        let node = &self.children.elements[id];

        let Element::Widget(widget) = &node.element else { unreachable!() };

        let children = Children {
            children: &node.children,
            elements: self.children.elements,
            attribute_reg: self.children.attribute_reg,
        };

        let attributes = &self.children.attribute_reg[id];
        let attributes = WidgetAttributes::new(self.children.elements, node.parent, attributes, self.children.attribute_reg);

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
    widget: RefMut<'a, Box<dyn Widget>>,
    pub attributes: WidgetAttributes<'a, 'bp>,
    children: Children<'a, 'bp>,
}

impl<'a, 'bp> WidgetRef<'a, 'bp> {
    pub(crate) fn new(
        id: ElementId,
        widget: RefMut<'a, Box<dyn Widget>>,
        attributes: WidgetAttributes<'a, 'bp>,
        children: Children<'a, 'bp>,
    ) -> Self {
        Self {
            id,
            widget,
            children,
            attributes,
        }
    }

    pub fn layout(&mut self, layout: &mut Layouts, constraints: Constraints) -> Size {
        let result = self.widget.layout(self.id, &mut self.children, &self.attributes, layout, constraints);
        layout.set_size(self.id, result);
        result.outer
    }

    pub fn position(&mut self, layout: &mut Layouts, pos: Pos) {
        layout.set_pos(self.id, pos);
        self.widget.position(self.id, &mut self.children, &self.attributes, layout, pos)
    }

    pub fn paint(mut self, frontend: &mut dyn Frontend, layout: &Layouts) {
        let desc = self.widget.describe();
        let region = layout[self.id].region();
        self.widget.paint(self.id, region, self.children, self.attributes, frontend, layout)
    }

    pub fn to<T: Widget>(&mut self) -> Option<&mut T> {
        let widget = self.widget.as_mut() as &mut dyn Any;
        widget.downcast_mut()
    }
}
