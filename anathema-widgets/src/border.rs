use anathema_core::layout::Layout;
use anathema_core::runtime::elements::{Children, Element};
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

    fn describe(&self) -> &str {
        "border"
    }
}

pub fn border() -> Box<dyn Element> {
    Box::new(Border)
}
