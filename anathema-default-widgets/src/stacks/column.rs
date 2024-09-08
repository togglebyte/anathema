use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::layout::Axis;
use crate::stacks::Stack;

pub struct Column(Stack);

impl Default for Column {
    fn default() -> Self {
        Self(Stack(Axis::Vertical))
    }
}

impl Widget for Column {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        self.0.layout(children, constraints, id, ctx)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        let x_offset = (ctx.inner_size.width / 2) as i32;

        children.for_each(|child, children| {
            let size = child.size();
            let child_width = size.width as i32;
            let x = x_offset - child_width / 2;

            let mut pos = ctx.pos;
            pos.x += x;

            child.position(children, pos, attribute_storage, ctx.viewport);
            ctx.pos.y += size.height as i32;
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn basic_column() {
        let tpl = "
            column
                text 'a'
                border
                    text 'b'
                text 'c'
        ";

        let expected = "
            ╔═══╗
            ║ a ║
            ║┌─┐║
            ║│b│║
            ║└─┘║
            ║ c ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 5)).instance().render_assert(expected);
    }
}
