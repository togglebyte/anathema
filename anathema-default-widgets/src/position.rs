use std::ops::ControlFlow;

use anathema::CommonVal;
use anathema_geometry::{Pos, Size};
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::{AttributeStorage, LayoutChildren, PositionChildren, Widget, WidgetId};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HorzEdge {
    Left(u32),
    Right(u32),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VertEdge {
    Top(u32),
    Bottom(u32),
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Placement {
    /// Widget is positioned relative to its parent
    Relative,
    /// Absolute position of a widget
    #[default]
    Absolute,
}

impl TryFrom<CommonVal<'_>> for Placement {
    type Error = ();

    fn try_from(value: CommonVal<'_>) -> Result<Self, Self::Error> {
        match value {
            CommonVal::Str(wrap) => match wrap {
                "relative" => Ok(Placement::Relative),
                "absolute" => Ok(Placement::Absolute),
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
        mut children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attribs = ctx.attribs.get(id);
        // TODO after the merge enable relative floats.
        //      This requires that we have the parent position
        //      when we do the layout.
        //      - TB 2024-05-02
        self.placement = Placement::Absolute; //attribs.get_c("placement").unwrap_or_default();

        self.horz_edge = match attribs.get_c("left") {
            Some(left) => HorzEdge::Left(left),
            None => match attribs.get_c("right") {
                Some(right) => HorzEdge::Right(right),
                None => HorzEdge::Left(0),
            },
        };

        self.vert_edge = match attribs.get_c("top") {
            Some(top) => VertEdge::Top(top),
            None => match attribs.get_c("bottom") {
                Some(bottom) => VertEdge::Bottom(bottom),
                None => VertEdge::Top(0),
            },
        };

        // Relative:
        // Position relative to parent means calculating a new constraint
        // based of the position of the top, left - the size
        //
        // Given a constraint of 10 x 10 and a left of 2 and a top of 3 it would
        // produce a new set of constraints at 8 x 7
        //
        // Absolute:
        // Position relative to the viewport,
        // Has no constraints

        let constraints = match self.placement {
            Placement::Relative => constraints,
            Placement::Absolute => ctx.viewport.constraints(),
        };

        let mut size = Size::ZERO;

        children.for_each(|child, children| {
            size = child.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });

        size.width = match self.horz_edge {
            HorzEdge::Left(left) => size.width + left as usize,
            HorzEdge::Right(right) => constraints.max_width() - right as usize,
        };

        size.height = match self.vert_edge {
            VertEdge::Top(top) => size.height + top as usize,
            VertEdge::Bottom(bottom) => constraints.max_height() - bottom as usize,
        };

        size
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        if let Placement::Absolute = self.placement {
            ctx.pos = Pos::ZERO;
        }

        children.for_each(|child, children| {
            match self.horz_edge {
                HorzEdge::Left(left) => ctx.pos.x += left as i32,
                HorzEdge::Right(right) => {
                    let offset = ctx.inner_size.width - child.size().width - right as usize;
                    ctx.pos.x = offset as i32;
                }
            }

            match self.vert_edge {
                VertEdge::Top(top) => ctx.pos.y += top as i32,
                VertEdge::Bottom(right) => {
                    let offset = ctx.inner_size.width - child.size().width - right as usize;
                    ctx.pos.x = offset as i32;
                }
            }
            child.position(children, ctx.pos, attribute_storage);
            ControlFlow::Break(())
        });
    }
}
