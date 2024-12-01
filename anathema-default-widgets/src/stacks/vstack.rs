use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, EvalContext, ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

use super::Stack;
use crate::layout::Axis;

pub struct VStack(Stack);

impl Default for VStack {
    fn default() -> Self {
        VStack(Stack(Axis::Vertical))
    }
}

impl Widget for VStack {
    fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut EvalContext<'_, '_, 'bp>,
    ) -> Size {
        self.0.layout(children, constraints, id, ctx)
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, 'bp>,
        attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.0.position(children, attributes, attribute_storage, ctx)
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        children.each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn vstack() {
        let tpl = "
            vstack
                border
                    text state.value
                for i in [2]
                    border
                        text i
        ";

        let expected_first = "
            ╔═══╗
            ║┌─┐║
            ║│0│║
            ║└─┘║
            ║┌─┐║
            ║│2│║
            ║└─┘║
            ╚═══╝
        ";

        let expected_second = "
            ╔═══╗
            ║┌─┐║
            ║│7│║
            ║└─┘║
            ║┌─┐║
            ║│2│║
            ║└─┘║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 6))
            .instance()
            .render_assert(expected_first)
            .with_state(|state| *state.value.to_mut() = 7)
            .render_assert(expected_second);
    }

    #[test]
    fn fixed_height() {
        let tpl = "
            vstack [height: 2]
                for i in [0, 1]
                    border
                        text i
        ";

        let expected = "
            ╔══╗
            ║┌┐║
            ║└┘║
            ╚══╝
        ";

        TestRunner::new(tpl, (2, 2)).instance().render_assert(expected);
    }

    #[test]
    fn fixed_width() {
        let tpl = "
            vstack [width: 7, height: 3]
                border
                    expand
                        text 'a'
        ";

        let expected = "
            ╔═══════╗
            ║┌─────┐║
            ║│a    │║
            ║└─────┘║
            ╚═══════╝
        ";

        TestRunner::new(tpl, (7, 3)).instance().render_assert(expected);
    }

    #[test]
    fn bottom_up_vstack() {
        let tpl = "
            vstack [direction: 'back']
                text 'b'
                border
                    text 'a'
        ";

        let expected = "
            ╔══════╗
            ║      ║
            ║┌─┐   ║
            ║│a│   ║
            ║└─┘   ║
            ║b     ║
            ╚══════╝
        ";

        TestRunner::new(tpl, (6, 5)).instance().render_assert(expected);
    }

    #[test]
    fn vstack_overflow() {
        let tpl = "
            vstack
                text 'a'
                text 'b'
                text 'c'
                text 'd'
        ";

        let expected = "
            ╔══════╗
            ║a     ║
            ║b     ║
            ╚══════╝
        ";

        let mut runner = TestRunner::new(tpl, (6, 2));
        let mut runner = runner.instance();
        runner.render_assert(expected);
    }
}
