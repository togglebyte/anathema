use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{
    AttributeStorage, ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId,
};

use crate::layout::{single_layout, Axis};

#[derive(Debug, Default)]
pub struct Expand;

impl Widget for Expand {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let mut size = single_layout(children, constraints, ctx);

        let attributes = ctx.attribute_storage.get(id);
        panic!();
        // match attributes.get("axis") {
        //     Some(Axis::Horizontal) => size.width = constraints.max_width(),
        //     Some(Axis::Vertical) => size.height = constraints.max_height(),
        //     None => {
        //         size.width = constraints.max_width();
        //         size.height = constraints.max_height();
        //     }
        // }

        size
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        children.each(|node, children| {
            node.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        children.each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn expand() {}
}
