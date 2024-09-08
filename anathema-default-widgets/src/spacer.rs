use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default)]
pub struct Spacer;

impl Widget for Spacer {
    fn layout<'bp>(
        &mut self,
        _children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        _id: WidgetId,
        _ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        Size::new(constraints.min_width, constraints.min_height)
    }

    fn paint<'bp>(
        &mut self,
        _: PaintChildren<'_, '_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PaintCtx<'_, SizePos>,
    ) {
        // The spacer widget has no children
    }

    fn position<'bp>(
        &mut self,
        _: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PositionCtx,
    ) {
        // The spacer widget has no children
    }
}
