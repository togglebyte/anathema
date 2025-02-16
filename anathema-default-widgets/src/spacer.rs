use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};
use anathema_widgets::error::Result;

#[derive(Debug, Default)]
pub struct Spacer;

impl Widget for Spacer {
    fn layout<'bp>(
        &mut self,
        _: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        _: WidgetId,
        _: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        Ok(Size::new(constraints.min_width, constraints.min_height))
    }

    fn paint<'bp>(
        &mut self,
        _: PaintChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PaintCtx<'_, SizePos>,
    ) {
        // The spacer widget has no children
    }

    fn position<'bp>(
        &mut self,
        _: PositionChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PositionCtx,
    ) {
        // The spacer widget has no children
    }
}
