use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

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
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        attributes: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        self.0.layout(children, constraints, attributes, ctx)
    }

    fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        self.0.position(children, attributes, attribute_storage, ctx)
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
                    text value
                for i in [0]
                    border
                        text i
        ";

        let expected_first = "
            ╔═══╗
            ║┌─┐║
            ║│0│║
            ║└─┘║
            ║┌─┐║
            ║│0│║
            ║└─┘║
            ╚═══╝
        ";

        let expected_second = "
            ╔═══╗
            ║┌─┐║
            ║│7│║
            ║└─┘║
            ║┌─┐║
            ║│0│║
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

        TestRunner::new(tpl, (6, 2)).instance().render_assert(expected);
    }
}
