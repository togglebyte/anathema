use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default)]
pub struct Spacer;

impl Widget for Spacer {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        Size::new(constraints.min_width, constraints.min_height)
    }

    fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PaintCtx<'_, SizePos>,
    ) {
        // The spacer widget has no children
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PositionCtx,
    ) {
        // The spacer widget has no children
    }
}
