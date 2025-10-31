use anathema_geometry::{Pos, Size};
use anathema_runtime::widgets::{Children, Layout, Widget};

#[derive(Debug, Default)]
pub struct Border;

impl Widget for Border {
    fn layout(&mut self, mut children: Children<'_, '_>, layout: &mut Layout) -> Size {
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

pub fn border() -> Box<dyn Widget> {
    Box::new(Border)
}
