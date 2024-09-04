use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::{HEIGHT, MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, WIDTH};

#[derive(Debug, Default)]
pub struct Container;

impl Widget for Container {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let mut size = Size::ZERO;

        let attribs = ctx.attribs.get(id);

        if let Some(width) = attribs.get_usize(WIDTH) {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attribs.get_usize(HEIGHT) {
            constraints.make_height_tight(height);
        }

        if let Some(width) = attribs.get_usize(MIN_WIDTH) {
            constraints.min_width = width;
        }

        if let Some(height) = attribs.get_usize(MIN_HEIGHT) {
            constraints.min_height = height;
        }

        if let Some(width) = attribs.get_usize(MAX_WIDTH) {
            constraints.set_max_width(width);
        }

        if let Some(height) = attribs.get_usize(MAX_HEIGHT) {
            constraints.set_max_height(height);
        }

        children.for_each(|child, children| {
            size = child.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });

        size.width = size.width.max(constraints.min_width);
        size.height = size.height.max(constraints.min_height);

        size
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        children.for_each(|child, children| {
            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn container() {
        let tpl = "
            container
                text 'a'
        ";

        let expected = "
            ╔══════╗
            ║a     ║
            ║      ║
            ╚══════╝
        ";

        TestRunner::new(tpl, (6, 2)).instance().render_assert(expected);
    }
}
