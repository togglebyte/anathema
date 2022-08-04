use crate::display::Size;

use crate::widgets::layout::vertical;
use crate::widgets::Attributes;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// A widget that lays out its children vertically.
/// ```text
/// ┌─┐
/// │1│
/// └─┘
/// ┌─┐
/// │2│
/// └─┘
/// ┌─┐
/// │3│
/// └─┘
/// ```
///
/// ```
/// use anathema::widgets::{VStack, Text, Widget, NodeId};
/// let mut vstack = VStack::new(None, None);
/// vstack.children.push(Text::with_text("1").into_container(NodeId::auto()));
/// vstack.children.push(Text::with_text("2").into_container(NodeId::auto()));
/// vstack.children.push(Text::with_text("3").into_container(NodeId::auto()));
/// ```
/// output:
/// ```text
/// 1
/// 2
/// 3
/// ```
#[derive(Debug)]
pub struct VStack {
    /// Children
    pub children: Vec<WidgetContainer>,
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Option<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Option<usize>,
}

impl VStack {
    /// Creates a new instance of a `VStack`
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self { children: Vec::new(), width: width.into(), height: height.into() }
    }
}

impl Widget for VStack {
    fn kind(&self) -> &'static str {
        "VStack"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, mut ctx: LayoutCtx) -> Size {
        if let Some(width) = self.width {
            ctx.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height {
            ctx.constraints.make_height_tight(height);
        }
        vertical::layout(&mut self.children, ctx)
    }

    fn position(&mut self, ctx: PositionCtx) {
        vertical::position(&mut self.children, ctx)
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        for child in self.children.iter_mut() {
            let ctx = ctx.sub_context(None);
            child.paint(ctx);
        }
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        self.children.iter_mut().collect()
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.children.push(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(pos) = self.children.iter().position(|c| c.id.eq(child_id)) {
            return Some(self.children.remove(pos));
        }

        None
    }

    fn update(&mut self, attr: Attributes) {
        if let Some(width) = attr.width() {
            self.width = Some(width);
        }
        if let Some(height) = attr.height() {
            self.height = Some(height);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::test_widget;
    use crate::widgets::{Border, BorderStyle, Sides, Text};

    fn test_column(col: impl Widget, expected: &str) {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        border.child = Some(col.into_container(NodeId::auto()));
        test_widget(border, expected);
    }

    #[test]
    fn column() {
        let mut vstack = VStack::new(None, None);
        vstack.add_child(Text::with_text("0").into_container(NodeId::auto()));
        vstack.add_child(Text::with_text("1").into_container(NodeId::auto()));
        vstack.add_child(Text::with_text("2").into_container(NodeId::auto()));
        test_column(
            vstack,
            r#"
            ┌───────┐
            │0      │
            │1      │
            │2      │
            └───────┘
            "#,
        );
    }

    #[test]
    fn fixed_size_column() {
        let mut vstack = VStack::new(None, 2);
        vstack.add_child(Text::with_text("0").into_container(NodeId::auto()));
        vstack.add_child(Text::with_text("1").into_container(NodeId::auto()));
        vstack.add_child(Text::with_text("2").into_container(NodeId::auto()));
        test_column(
            vstack,
            r#"
            ┌───────┐
            │0      │
            │1      │
            │       │
            └───────┘
            "#,
        );
    }
}
