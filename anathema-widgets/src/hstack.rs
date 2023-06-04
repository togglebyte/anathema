use anathema_render::Size;

use super::{
    LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize,
};
// use crate::layout::horizontal; 

/// A widget that lays out its children horizontally.
/// ```text
/// ┌─┐┌─┐┌─┐┌─┐
/// │1││2││3││4│
/// └─┘└─┘└─┘└─┘
/// ```
///
/// ```ignore
/// use anathema_widgets::{HStack, Text, Widget, NodeId};
/// let mut hstack = HStack::new(None, None);
/// hstack.children.push(Text::with_text("1").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("2").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("3").into_container(NodeId::anon()));
/// hstack.children.push(Text::with_text("4").into_container(NodeId::anon()));
/// ```
/// output:
/// ```text
/// 1234
/// ```
#[derive(Debug)]
pub struct HStack {
    /// If a width is provided then the layout constraints will be tight for width
    pub width: Option<usize>,
    /// If a height is provided then the layout constraints will be tight for height
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
}

impl HStack {
    /// Create a new instance of an `HStack`.
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            min_width: None,
            min_height: None,
        }
    }
}

// impl Widget for HStack {
//     fn kind(&self) -> &'static str {
//         "HStack"
//     }

//     fn as_any_ref(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }

//     fn layout<'gen: 'ctx, 'ctx>(&mut self, mut ctx: LayoutCtx<'gen, 'ctx>, children: &mut Children<'gen>) -> Size {
//         if let Some(width) = self.width {
//             ctx.constraints.make_width_tight(width);
//         }
//         if let Some(height) = self.height {
//             ctx.constraints.make_height_tight(height);
//         }
//         if let Some(min_width) = self.min_width {
//             ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
//         }
//         if let Some(min_height) = self.min_height {
//             ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
//         }

//         horizontal::layout(ctx, false, children)
//     }

//     fn position<'gen: 'ctx, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
//         horizontal::position(ctx, children)
//     }

//     fn paint<'gen: 'ctx, 'ctx>(&mut self, mut ctx: PaintCtx<'_, WithSize>, children: &mut [WidgetContainer<'gen>]) {
//         let len = children.len();
//         for child in children {
//             let ctx = ctx.sub_context(None);
//             child.paint(ctx);
//         }
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
