use std::ops::ControlFlow;

use anathema_geometry::{Pos, Size};
use anathema_value_resolver::{AttributeStorage, ValueKind};
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::{BOTTOM, LEFT, RIGHT, TOP};

const RELATIVE: &str = "relative";
const ABSOLUTE: &str = "absolute";
const PLACEMENT: &str = "placement";

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HorzEdge {
    Left(i32),
    Right(i32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VertEdge {
    Top(i32),
    Bottom(i32),
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Placement {
    /// Widget is positioned relative to its parent
    #[default]
    Relative,
    /// Absolute position of a widget
    Absolute,
}

impl TryFrom<&ValueKind<'_>> for Placement {
    type Error = ();

    fn try_from(value: &ValueKind<'_>) -> Result<Self, Self::Error> {
        match value {
            ValueKind::Str(wrap) => match wrap.as_ref() {
                RELATIVE => Ok(Placement::Relative),
                ABSOLUTE => Ok(Placement::Absolute),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Position {
    horz_edge: HorzEdge,
    vert_edge: VertEdge,
    placement: Placement,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            horz_edge: HorzEdge::Left(0),
            vert_edge: VertEdge::Top(0),
            placement: Placement::Relative,
        }
    }
}

impl Widget for Position {
    fn floats(&self) -> bool {
        true
    }

    fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
        let attribs = ctx.attribute_storage.get(id);
        self.placement = attribs.get_as::<Placement>(PLACEMENT).unwrap_or_default();

        self.horz_edge = match attribs.get_as::<i32>(LEFT) {
            Some(left) => HorzEdge::Left(left),
            None => match attribs.get_as::<i32>(RIGHT) {
                Some(right) => HorzEdge::Right(right),
                None => HorzEdge::Left(0),
            },
        };

        self.vert_edge = match attribs.get_as::<i32>(TOP) {
            Some(top) => VertEdge::Top(top),
            None => match attribs.get_as::<i32>(BOTTOM) {
                Some(bottom) => VertEdge::Bottom(bottom),
                None => VertEdge::Top(0),
            },
        };

        let size = constraints.max_size();

        _ = children.each(ctx, |ctx, child, children| {
            // size is determined by the constraint
            _ = child.layout(children, ctx.viewport.constraints(), ctx)?;
            Ok(ControlFlow::Break(()))
        })?;

        Ok(size)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        if let Placement::Absolute = self.placement {
            ctx.pos = Pos::ZERO;
        }

        _ = children.each(|child, children| {
            match self.horz_edge {
                HorzEdge::Left(left) => ctx.pos.x += left,
                HorzEdge::Right(right) => {
                    let offset = ctx.pos.x + ctx.inner_size.width as i32 - child.size().width as i32 - right;
                    ctx.pos.x = offset;
                }
            }

            match self.vert_edge {
                VertEdge::Top(top) => ctx.pos.y += top,
                VertEdge::Bottom(bottom) => {
                    let offset = ctx.pos.y + ctx.inner_size.height as i32 - child.size().height as i32 - bottom;
                    ctx.pos.y = offset;
                }
            }

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
        _ = children.each(|child, children| {
            let mut ctx = ctx.to_unsized();
            ctx.clip = None;
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Continue(())
        });
    }
}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn position_top_left() {
        let tpl = "
            position [top: 0, left: 0]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║hi  ║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_top() {
        let tpl = "
            position [top: 1]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║    ║
            ║hi  ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_top_right() {
        let tpl = "
            position [top: 1, right: 0]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║    ║
            ║  hi║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_right() {
        let tpl = "
            position [right: 0]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║  hi║
            ║    ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_bottom_right() {
        let tpl = "
            position [placement: 'relative', bottom: 0, right: 0]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║    ║
            ║  hi║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_bottom() {
        let tpl = "
            position [bottom: 0]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║    ║
            ║hi  ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }

    #[test]
    fn position_bottom_left() {
        let tpl = "
            position [bottom: 0, left: 1]
                text 'hi'
            ";

        let expected = "
            ╔════╗
            ║    ║
            ║ hi ║
            ╚════╝
        ";

        TestRunner::new(tpl, (4, 2)).instance().render_assert(expected);
    }
}
