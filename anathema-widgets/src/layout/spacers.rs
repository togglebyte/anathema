use anathema_render::Size;

use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::{Axis, Spacer};

pub fn layout(ctx: &mut LayoutCtx<'_, '_, '_>, axis: Axis) -> Result<Size> {
    let mut final_size = Size::ZERO;
    let count = ctx
        .children
        .iter()
        .filter(|c| c.kind() == Spacer::KIND)
        .count();

    if count == 0 {
        return Ok(final_size);
    }

    match axis {
        Axis::Horizontal => ctx.constraints.max_width /= count,
        Axis::Vertical => ctx.constraints.max_height /= count,
    };

    ctx.constraints.min_width = ctx.constraints.max_width;
    ctx.constraints.min_height = ctx.constraints.max_height;

    for spacer in ctx.children.iter_mut().filter(|c| c.kind() == Spacer::KIND) {
        let size = spacer.layout(ctx.constraints, ctx.values, ctx.lookup)?;

        match axis {
            Axis::Horizontal => {
                final_size.width += size.width;
                final_size.height = final_size.height.max(size.height);
            }
            Axis::Vertical => {
                final_size.height += size.height;
                final_size.width = final_size.width.max(size.width);
            }
        }
    }

    Ok(final_size)
}
