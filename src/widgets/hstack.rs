use crate::display::Size;

use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::widgets::layout::horizontal;
use crate::widgets::Attributes;

/// A widget that lays out its children horizontally.
/// ```text
/// ┌─┐┌─┐┌─┐┌─┐
/// │1││2││3││4│
/// └─┘└─┘└─┘└─┘
/// ```
///
/// ```
/// use anathema::widgets::{HStack, Text, Widget, NodeId};
/// let mut hstack = HStack::new(None, None);
/// hstack.children.push(Text::with_text("1").into_container(NodeId::auto()));
/// hstack.children.push(Text::with_text("2").into_container(NodeId::auto()));
/// hstack.children.push(Text::with_text("3").into_container(NodeId::auto()));
/// hstack.children.push(Text::with_text("4").into_container(NodeId::auto()));
/// ```
/// output:
/// ```text
/// 1234
/// ```
#[derive(Debug)]
pub struct HStack {
    /// Children
    pub children: Vec<WidgetContainer>,
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Option<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Option<usize>,
}

impl HStack {
    /// Create a new instance of an `HStack`.
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self { children: Vec::new(), width: width.into(), height: height.into() }
    }
}

impl Widget for HStack {
    fn kind(&self) -> &'static str {
        "HStack"
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
        horizontal::layout(&mut self.children, ctx)
    }

    fn position(&mut self, ctx: PositionCtx) {
        horizontal::position(&mut self.children, ctx)
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
