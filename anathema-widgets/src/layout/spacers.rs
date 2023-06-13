use anathema_render::Size;

use super::Constraints;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Axis, Spacer, WidgetContainer};

pub fn layout(ctx: &mut LayoutCtx<'_, '_, '_>, axis: Axis) -> Result<Size> {
    let mut size = Size::ZERO;
    let count = ctx
        .children
        .iter()
        .filter(|c| c.kind() == Spacer::KIND)
        .count();

    if count == 0 {
        return Ok(size);
    }

    match axis {
        Axis::Horizontal => ctx.constraints.max_width /= count,
        Axis::Vertical => ctx.constraints.max_height /= count,
    };

    ctx.constraints.min_width = ctx.constraints.max_width;
    ctx.constraints.min_height = ctx.constraints.max_height;

    for spacer in ctx.children.iter_mut().filter(|c| c.kind() == Spacer::KIND) {
        // Ignore all widgets that aren't spacers
        if spacer.kind() != Spacer::KIND {
            continue;
        }

        let s = spacer.layout(ctx.constraints, ctx.values, ctx.lookup)?;

        match axis {
            Axis::Horizontal => {
                size.width += s.width;
                size.height = size.height.max(s.height);
            }
            Axis::Vertical => {
                size.height += s.height;
                size.width = size.width.max(s.width);
            }
        }
    }

    Ok(size)
}
