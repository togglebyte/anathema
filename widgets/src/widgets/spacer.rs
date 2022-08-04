use display::Size;

use crate::attributes::Attributes;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// Expand to fill in all available space.
///
/// Unlike the [`Expanded`](crate::Expanded) widget, the `Spacer` only works inside
/// [`HStack`](crate::HStack) and [`VStack`](crate::VStack), and flows in the
/// direction of the stack.
///
/// In an `HStack` the spacer will always expand to have the same height as the child with the most
/// height.
///
/// In an `VStack` the spacer will always expand to have the same width as the child with the most
/// width.
#[derive(Debug)]
pub struct Spacer;

impl Spacer {
    /// Widget name
    pub const KIND: &'static str = "Spacer";
}

impl Widget for Spacer {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        debug_assert!(
            ctx.constraints.is_width_tight() && ctx.constraints.is_height_tight(),
            "the layout context needs to be tight for a spacer"
        );
        Size::new(ctx.constraints.min_width, ctx.constraints.min_height)
    }

    fn position(&mut self, _: PositionCtx) {}

    fn paint(&mut self, _ctx: PaintCtx<'_, WithSize>) {}

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        vec![]
    }

    fn add_child(&mut self, _widget: WidgetContainer) {}

    fn remove_child(&mut self, _child_id: &NodeId) -> Option<WidgetContainer> {
        None
    }

    fn update(&mut self, _: Attributes) {}
}
