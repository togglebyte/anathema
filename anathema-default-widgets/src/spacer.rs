use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Default)]
pub struct Spacer;

impl Widget for Spacer {
    fn layout<'bp>(
        &mut self,
        _: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        _: WidgetId,
        _: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        Ok(Size::new(constraints.min_width, constraints.min_height))
    }

    fn paint<'bp>(
        &mut self,
        _: PaintChildren<'_, 'bp>,
        _: WidgetId,
        _: &AttributeStorage<'bp>,
        _: PaintCtx<'_, SizePos>,
    ) {
        // The spacer widget has no children
    }

    fn position<'bp>(&mut self, _: PositionChildren<'_, 'bp>, _: WidgetId, _: &AttributeStorage<'bp>, _: PositionCtx) {
        // The spacer widget has no children
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn hstack_spacer_nospace() {
        let tpl = "
        border
            container [width:12]
                hstack
                    spacer
                    text 'this is text'
        ";

        let expected = "
            ╔══════════════╗
            ║┌────────────┐║
            ║│this is text│║
            ║└────────────┘║
            ║              ║
            ╚══════════════╝
        ";

        let mut runner = TestRunner::new(tpl, (14, 4));
        let mut runner = runner.instance();
        runner.render_assert(expected);
    }

    #[test]
    fn vstack_spacer_nospace() {
        let tpl = "
        border
            container [height:1]
                vstack
                    spacer
                    text 'this is text'
        ";

        let expected = "
            ╔═════════════════╗
            ║┌────────────┐   ║
            ║│this is text│   ║
            ║└────────────┘   ║
            ║                 ║
            ╚═════════════════╝
        ";

        let mut runner = TestRunner::new(tpl, (17, 4));
        let mut runner = runner.instance();
        runner.render_assert(expected);
    }
}
