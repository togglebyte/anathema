use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use super::Stack;
use crate::layout::Axis;

pub struct HStack(Stack);

impl Default for HStack {
    fn default() -> Self {
        HStack(Stack(Axis::Horizontal))
    }
}

impl Widget for HStack {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        attributes: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        self.0.layout(children, constraints, attributes, ctx)
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.0.position(children, attributes, attribute_storage, ctx)
    }
}
