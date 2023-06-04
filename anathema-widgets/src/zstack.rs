use anathema_render::Size;

use super::{
    LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize,
};
use crate::layout::stacked;

/// Unlike the [`HStack`](crate::HStack) or the [`VStack`](crate::VStack) the [`ZStack`] draws the
/// children on top of each other.
///
/// This makes it possible to draw widgets on top of other widgets.
///
/// An example adding a title to a border
/// ```ignore
/// use anathema_widgets::{ZStack, Position, Border, Text, Widget, NodeId, HorzEdge, VertEdge};
///
/// let mut zstack = ZStack::new(None, None).into_container(NodeId::anon());
///
/// // Border
/// let mut border = Border::thin(20, 5).into_container(NodeId::anon());
/// border.add_child(Text::with_text("Here is some text").into_container(NodeId::anon()));
/// zstack.add_child(border);
///
/// // Title
/// let mut position = Position::new(HorzEdge::Left(1), VertEdge::Top(0)).into_container(NodeId::anon());
/// position.add_child(Text::with_text("] Title [").into_container(NodeId::anon()));
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
        Self {
            width: width.into(),
            height: height.into(),
            min_width: None,
            min_height: None,
        }
    }
}

// impl Widget for ZStack {
//     fn kind(&self) -> &'static str {
//         "ZStack"
//     }

//     fn as_any_ref(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }

//     fn layout(&mut self, ctx: LayoutCtx, children: &mut Vec<WidgetContainer<'_>>) -> Size {
//         panic!()
//         // if let Some(min_width) = self.min_width {
//         //     ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
//         // }
//         // if let Some(min_height) = self.min_height {
//         //     ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
//         // }
//         // if let Some(width) = self.width {
//         //     ctx.constraints.make_width_tight(width);
//         // }
//         // if let Some(height) = self.height {
//         //     ctx.constraints.make_height_tight(height);
//         // }
//         // stacked::layout(&mut self.children, ctx)
//     }

//     fn position(&mut self, ctx: PositionCtx) {
//         panic!()
//         // stacked::position(&mut self.children, ctx)
//     }

//     fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
//         panic!()
//         // for child in self.children.iter_mut() {
//         //     let ctx = ctx.sub_context(None);
//         //     child.paint(ctx);
//         // }
//     }

//     // fn update(&mut self, ctx: UpdateCtx) {
//     //     if let Some(width) = ctx.attributes.width() {
//     //         self.width = Some(width);
//     //     }
//     //     if let Some(height) = ctx.attributes.height() {
//     //         self.height = Some(height);
//     //     }
//     // }
// }
