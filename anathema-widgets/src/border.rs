use anathema_core::elements::Element;
use anathema_core::layout::Layout;
use anathema_core::nodes::{Children, InsertNode};
use anathema_geometry::{Pos, Size};

#[derive(Debug, Default)]
pub struct Border;

impl Element for Border {
    fn layout(&mut self, mut children: Children<'_>, layout: &mut Layout) -> Size {
        let mut size = children.next().map(|child| child.layout(layout)).unwrap_or(Size::ZERO);
        size
    }

    fn position(&mut self) -> Pos {
        todo!()
    }

    fn paint(&mut self) {
        todo!()
    }
}

pub fn border() -> InsertNode {
    Border.into()
}
