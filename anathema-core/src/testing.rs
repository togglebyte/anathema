use anathema_geometry::{Pos, Size};

use crate::elements::Element;
use crate::layout::Layout;
use crate::nodes::{Children, InsertNode};

pub struct TestElement(String);

impl Element for TestElement {
    fn layout(&mut self, children: Children<'_>, layout: &mut Layout) -> Size {
        let mut size = Size::new(self.0.len() as u16, 1);

        for mut child in children {
            let child_size = child.layout(layout);
            size.width = size.width.max(child_size.width);
            size.height += child_size.height;
        }

        size
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self) {
        todo!()
    }

    fn describe(&self) -> &str {
        &self.0
    }
}

pub(crate) fn test_el(value: impl Into<String>) -> InsertNode {
    let el = TestElement(value.into());
    InsertNode::new(el)
}
