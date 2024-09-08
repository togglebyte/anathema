use std::ops::ControlFlow;

use anathema_geometry::{Pos, Size};
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

use crate::layout::alignment::{Alignment, ALIGNMENT};

#[derive(Default)]
pub struct Align;

impl Widget for Align {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        _: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        children.for_each(|widget, children| {
            let _ = widget.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });

        constraints.max_size()
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        let attributes = attribute_storage.get(id);
        let alignment = attributes.get(ALIGNMENT).unwrap_or_default();

        children.for_each(|child, children| {
            let width = ctx.inner_size.width as i32;
            let height = ctx.inner_size.height as i32;
            let child_width = child.size().width as i32;
            let child_height = child.size().height as i32;

            let child_offset = match alignment {
                Alignment::TopLeft => Pos::ZERO,
                Alignment::Top => Pos::new(width / 2 - child_width / 2, 0),
                Alignment::TopRight => Pos::new(width - child_width, 0),
                Alignment::Right => Pos::new(width - child_width, height / 2 - child_height / 2),
                Alignment::BottomRight => Pos::new(width - child_width, height - child_height),
                Alignment::Bottom => Pos::new(width / 2 - child_width / 2, height - child_height),
                Alignment::BottomLeft => Pos::new(0, height - child_height),
                Alignment::Left => Pos::new(0, height / 2 - child_height / 2),
                Alignment::Centre => Pos::new(width / 2 - child_width / 2, height / 2 - child_height / 2),
            };

            child.position(children, ctx.pos + child_offset, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn top_left() {
        let tpl = "
            align [alignment: 'top_left']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║x  ║
            ║   ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn top() {
        let tpl = "
            align [alignment: 'top']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║ x ║
            ║   ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn top_right() {
        let tpl = "
            align [alignment: 'top_right']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║  x║
            ║   ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn right() {
        let tpl = "
            align [alignment: 'right']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║  x║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn bottom_right() {
        let tpl = "
            align [alignment: 'bottom_right']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║   ║
            ║  x║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn bottom() {
        let tpl = "
            align [alignment: 'bottom']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║   ║
            ║ x ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn bottom_left() {
        let tpl = "
            align [alignment: 'bottom_left']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║   ║
            ║x  ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn left() {
        let tpl = "
            align [alignment: 'left']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║x  ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn centre() {
        let tpl = "
            align [alignment: 'centre']
                text 'x'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║ x ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }
}
