use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{LayoutForEach, PositionChildren, Widget, WidgetId};

use crate::{HEIGHT, MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, WIDTH};

#[derive(Debug, Default)]
pub struct Container;

impl Widget for Container {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let mut size = Size::ZERO;

        let attribs = ctx.attribute_storage.get(id);

        if let Some(width) = attribs.get_as::<u16>(WIDTH) {
            constraints.make_width_tight(width);
        }

        if let Some(height) = attribs.get_as::<u16>(HEIGHT) {
            constraints.make_height_tight(height);
        }

        if let Some(width) = attribs.get_as::<u16>(MIN_WIDTH) {
            constraints.min_width = width;
        }

        if let Some(height) = attribs.get_as::<u16>(MIN_HEIGHT) {
            constraints.min_height = height;
        }

        if let Some(width) = attribs.get_as::<u16>(MAX_WIDTH) {
            constraints.set_max_width(width);
        }

        if let Some(height) = attribs.get_as::<u16>(MAX_HEIGHT) {
            constraints.set_max_height(height);
        }

        children.each(ctx, |ctx, child, children| {
            size = child.layout(children, constraints, ctx)?.into();
            Ok(ControlFlow::Break(()))
        })?;

        size.width = size.width.max(constraints.min_width);
        size.height = size.height.max(constraints.min_height);

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        children.each(|child, children| {
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
