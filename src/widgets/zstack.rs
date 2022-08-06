use crate::display::Size;

use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::widgets::layout::stacked;
use crate::widgets::Attributes;

/// Unlike the [`HStack`](crate::HStack) or the [`VStack`](crate::VStack) the [`ZStack`] draws the
/// children on top of each other.
///
/// This makes it possible to draw widgets on top of other widgets.
///
/// An example adding a title to a border
/// ```
/// use anathema::widgets::{ZStack, Position, Border, Text, Widget, NodeId, HorzEdge, VertEdge};
///
/// let mut zstack = ZStack::new(None, None).into_container(NodeId::auto());
///
/// // Border
/// let mut border = Border::thin(20, 5).into_container(NodeId::auto());
/// border.add_child(Text::with_text("Here is some text").into_container(NodeId::auto()));
/// zstack.add_child(border);
///
/// // Title
/// let mut position = Position::new(HorzEdge::Left(1), VertEdge::Top(0)).into_container(NodeId::auto());
/// position.add_child(Text::with_text("] Title [").into_container(NodeId::auto()));
/// zstack.add_child(position);
/// ```
/// output
/// ```text
/// ┌] Title [─────────┐
/// │Here is some text │
/// │                  │
/// │                  │
/// └──────────────────┘
/// ```
///
/// Note that widgets are drawn in the order they are inserted.
/// To make something like a dialogue box appear on top it would have to be the last child of the
/// `ZStack`.
#[derive(Debug)]
pub struct ZStack {
    /// Child widgets
    pub children: Vec<WidgetContainer>,
    /// Width
    pub width: Option<usize>,
    /// Height
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
}

impl ZStack {
    /// Create a new instance of a `ZStack`
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self { children: Vec::new(), width: width.into(), height: height.into(), min_width: None, min_height: None }
    }
}

impl Widget for ZStack {
    fn kind(&self) -> &'static str {
        "ZStack"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, mut ctx: LayoutCtx) -> Size {
        if let Some(min_width) = self.min_width {
            ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        }
        if let Some(min_width) = self.min_width {
            ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        }
        if let Some(min_height) = self.min_height {
            ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        }
        if let Some(width) = self.width {
            ctx.constraints.make_width_tight(width);
        }
        if let Some(height) = self.height {
            ctx.constraints.make_height_tight(height);
        }
        stacked::layout(&mut self.children, ctx)
    }

    fn position(&mut self, ctx: PositionCtx) {
        stacked::position(&mut self.children, ctx)
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
