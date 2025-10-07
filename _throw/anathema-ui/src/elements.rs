use anathema_geometry::{Pos, Size};

use crate::nodes::Nodes;

pub trait Element {
    fn layout(&mut self) -> Size;
    fn position(&mut self) -> Pos;
    fn paint(&mut self);
}

pub struct ElementBuilder<'a, T> {
    element: T,
    nodes: &'a mut Nodes
}

impl<'a, T> ElementBuilder<'a, T> {
    fn add_child(self, child: impl Element) -> Self {
        self
    }

    // fn to_element(self) -> 
}

// -----------------------------------------------------------------------------
//   - The following code should be moved / thrown out -
// -----------------------------------------------------------------------------

pub struct Text {
    text: String,
}

impl Element for Text {
    fn layout(&mut self) -> Size {
        todo!()
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self) {
        todo!()
    }
}

pub fn text(text: String) -> Text {
    Text {
        text
    }
}

pub struct VStack {
}

impl Element for VStack {
    fn layout(&mut self) -> Size {
        todo!()
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self) {
        todo!()
    }
}
