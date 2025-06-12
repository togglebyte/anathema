use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::layout::{Axis, single_layout};

#[derive(Debug, Default)]
pub struct Expand;

impl Widget for Expand {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let mut size = single_layout(children, constraints, ctx)?;

        let attributes = ctx.attribute_storage.get(id);
        match attributes.get_as::<Axis>("axis") {
            Some(Axis::Horizontal) => size.width = constraints.max_width(),
            Some(Axis::Vertical) => size.height = constraints.max_height(),
            None => {
                size.width = constraints.max_width();
                size.height = constraints.max_height();
            }
        }

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        _ = children.each(|node, children| {
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
        _ = children.each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn hstack_expand_nospace() {
        let tpl = "
        border
            container [width:12]
                hstack
                    expand
                    text \"this is text\"
        ";

        let expected = "
            ╔══════════════╗
            ║┌────────────┐║
            ║│this is text│║
            ║│            │║
            ║└────────────┘║
            ╚══════════════╝
        ";

        let mut runner = TestRunner::new(tpl, (14, 4));
        let mut runner = runner.instance();
        runner.render_assert(expected);
    }

    #[test]
    fn vstack_expand_nospace() {
        let tpl = "
        border
            container [height:1]
                vstack
                    expand
                    text \"this is text\"
        ";

        let expected = "
            ╔═════════════════╗
            ║┌───────────────┐║
            ║│this is text   │║
            ║└───────────────┘║
            ║                 ║
            ╚═════════════════╝
        ";

        let mut runner = TestRunner::new(tpl, (17, 4));
        let mut runner = runner.instance();
        runner.render_assert(expected);
    }
}
