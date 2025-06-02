use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{LayoutForEach, PositionChildren, Widget, WidgetId};

#[derive(Default)]
pub struct ZStack;

impl Widget for ZStack {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        _: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let mut size = Size::ZERO;
        _ = children.each(ctx, |ctx, child, children| {
            let child_size = Size::from(child.layout(children, constraints, ctx)?);
            size.width = size.width.max(child_size.width);
            size.height = size.height.max(child_size.height);
            Ok(ControlFlow::Continue(()))
        })?;
        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        _ = children.each(|child, children| {
            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn zstack() {
        let tpl = "
            zstack
                text '333'
                text '22'
                text '1'
        ";

        let expected = "
            ╔═══╗
            ║123║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 1)).instance().render_assert(expected);
    }
}
