use std::ops::ControlFlow;

use anathema_geometry::{Pos, Region, Size};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};
use anathema_widgets::error::Result;

use crate::{BOTTOM, LEFT, RIGHT, TOP};

const PADDING: &str = "padding";

#[derive(Default)]
struct PaddingValues {
    top: u16,
    right: u16,
    bottom: u16,
    left: u16,
}

impl PaddingValues {
    fn size(&self) -> Size {
        Size::new(self.left + self.right, self.top + self.bottom)
    }
}

#[derive(Default)]
pub struct Padding(PaddingValues);

impl Widget for Padding {
    fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let attributes = ctx.attribute_storage.get(id);
        let mut size = Size::ZERO;
        let padding = attributes.get_as::<u16>(PADDING).unwrap_or(0);

        self.0.top = attributes
            .get_as::<usize>(TOP)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(padding);
        self.0.right = attributes
            .get_as::<usize>(RIGHT)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(padding);
        self.0.bottom = attributes
            .get_as::<usize>(BOTTOM)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(padding);
        self.0.left = attributes
            .get_as::<usize>(LEFT)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(padding);

        let padding_size = self.0.size();

        children.each(ctx, |ctx, child, children| {
            let mut child_constraints = constraints;
            child_constraints.sub_max_width(padding_size.width);
            child_constraints.sub_max_height(padding_size.height);
            let mut child_size = Size::from(child.layout(children, child_constraints, ctx)?);
            child_size += padding_size;
            size.width = child_size.width.max(size.width);
            size.height = child_size.height.max(size.height);

            Ok(ControlFlow::Break(()))
        })?;

        size.width = constraints.min_width.max(size.width).min(constraints.max_width());
        size.height = constraints.min_height.max(size.height).min(constraints.max_height());

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        children.each(|child, children| {
            ctx.pos.y += self.0.top as i32;
            ctx.pos.x += self.0.left as i32;

            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        children.each(|child, children| {
            let mut ctx = ctx.to_unsized();
            if let Some(clip) = ctx.clip.as_mut() {
                clip.from.x += self.0.left as i32;
                clip.from.y += self.0.top as i32;
                clip.to.x -= self.0.right as i32;
                clip.to.y -= self.0.bottom as i32;
            }
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Break(())
        });
    }

    fn inner_bounds(&self, mut pos: Pos, mut size: Size) -> Region {
        pos.x += self.0.left as i32;
        pos.y += self.0.top as i32;
        size.width = size.width.saturating_sub(self.0.right);
        size.height = size.height.saturating_sub(self.0.bottom);
        Region::from((pos, size))
    }
}

#[cfg(test)]
mod test {

    use crate::testing::TestRunner;

    #[test]
    fn padding_all() {
        let tpl = "
            padding [padding: 1]
                text 'a'
        ";

        let expected = "
            ╔═══╗
            ║   ║
            ║ a ║
            ║   ║
            ╚═══╝
        ";

        TestRunner::new(tpl, (3, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_top_inclusive() {
        let tpl = "
            padding [padding: 1, top: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║    ║
            ║    ║
            ║ a  ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 4)).instance().render_assert(expected);
    }

    #[test]
    fn padding_top() {
        let tpl = "
            padding [top: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║    ║
            ║    ║
            ║a   ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_right_inclusive() {
        let tpl = "
            padding [padding: 1, right: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║    ║
            ║ a  ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_right() {
        let tpl = "
            padding [right: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║a   ║
            ║    ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_bottom_inclusive() {
        let tpl = "
            padding [padding: 1, bottom: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║    ║
            ║ a  ║
            ║    ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 4)).instance().render_assert(expected);
    }

    #[test]
    fn padding_bottom() {
        let tpl = "
            padding [bottom: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║a   ║
            ║    ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_left_inclusive() {
        let tpl = "
            padding [padding: 1, left: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║    ║
            ║  a ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }

    #[test]
    fn padding_left() {
        let tpl = "
            padding [left: 2]
                text 'a'
        ";

        let expected = "
            ╔════╗
            ║  a ║
            ║    ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 3)).instance().render_assert(expected);
    }
}
