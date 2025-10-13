use anathema_geometry::{Pos, Size};

use crate::layout::Layout;
use crate::runtime::widgets::iter::Children;
use crate::runtime::widgets::Widget;

#[derive(Debug, Default)]
pub struct TestWidget(String);

impl Widget for TestWidget {
    fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size {
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
        if self.0.is_empty() {
            return "<test>";
        }
        &self.0
    }
}

// pub(crate) fn test_widget(value: impl Into<String>) -> InsertNode {
//     let el = TestWidget(value.into());
//     InsertNode::from(el)
// }
