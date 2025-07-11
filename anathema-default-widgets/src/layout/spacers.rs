use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::LayoutForEach;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx};

use super::Axis;

/// Layout spacers.
/// This is different to [`SpacerLayout`] which
/// does the layout of the child of a single [`Spacer`],
/// whereas this does the layout of multiple [`Spacer`]s
/// inside already evaluated children.
pub fn layout_all_spacers<'bp>(
    nodes: &mut LayoutForEach<'_, 'bp>,
    constraints: Constraints,
    axis: Axis,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Result<Size> {
    let mut final_size = Size::ZERO;
    let mut count = 0;

    _ = nodes.each(ctx, |_, node, _| {
        if node.ident == "spacer" {
            count += 1;
        }
        Ok(ControlFlow::Continue(()))
    })?;

    if count == 0 {
        return Ok(final_size);
    }

    let max = match axis {
        Axis::Horizontal => constraints.max_width(),
        Axis::Vertical => constraints.max_height(),
    };

    let mut overflow = max % count;

    _ = nodes.each(ctx, |ctx, node, children| {
        if node.ident != "spacer" {
            return Ok(ControlFlow::Continue(()));
        }

        // This is a bit gross
        let overflow = if overflow > 0 {
            overflow -= 1;
            1
        } else {
            0
        };

        let constraints = match axis {
            Axis::Horizontal => constraints.div_assign_max_width(count, overflow),
            Axis::Vertical => constraints.div_assign_max_height(count, overflow),
        };

        let size = Size::from(node.layout(children, constraints, ctx)?);

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

        Ok(ControlFlow::Continue(()))
    })?;

    Ok(final_size)
}
