use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::layout::Axis;
use crate::stacks::Stack;

pub struct Row(Stack);

impl Default for Row {
    fn default() -> Self {
        Self(Stack(Axis::Horizontal))
    }
}

impl Widget for Row {
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
        let y_offset = (ctx.inner_size.height / 2) as i32;

        children.for_each(|child, children| {
            let size = child.size();
            let child_height = size.height as i32;
            let y = y_offset - child_height / 2;

            let mut pos = ctx.pos;
            pos.y += y;
            child.position(children, pos, attribute_storage, ctx.viewport);
            ctx.pos.x += size.width as i32;
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn basic_row() {
        let tpl = "
            row
                text 'a'
                border
                    text 'b'
        ";

        let expected = "
            ╔════╗
            ║ ┌─┐║
            ║a│b│║
            ║ └─┘║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }
}
