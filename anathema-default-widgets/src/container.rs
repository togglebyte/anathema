use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default)]
pub struct Container;

impl Widget for Container {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        let mut size = Size::ZERO;

        let attribs = ctx.attribs.get(id);

        if let Some(width @ 0..=i64::MAX) = attribs.get("width") {
            constraints.make_width_tight(width as usize);
        }

        if let Some(height @ 0..=i64::MAX) = attribs.get("height") {
            constraints.make_height_tight(height as usize);
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
            child.position(children, ctx.pos, attribute_storage);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    use super::*;

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
