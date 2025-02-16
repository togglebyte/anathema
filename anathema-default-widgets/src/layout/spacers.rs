use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx};
use anathema_widgets::LayoutForEach;
use anathema_widgets::error::Result;

use super::Axis;

/// Layout spacers.
/// This is different to [`SpacerLayout`] which
/// does the layout of the child of a single [`Spacer`],
/// whereas this does the layout of multiple [`Spacer`]s
/// inside already evaluated children.
pub fn layout_all_spacers<'bp>(
    nodes: &mut LayoutForEach<'_, 'bp>,
    mut constraints: Constraints,
    axis: Axis,
    ctx: &mut LayoutCtx<'_, 'bp>,
) -> Result<Size> {
    let mut final_size = Size::ZERO;
    let mut count = 0;
    nodes.each(ctx, |_, node, _| {
        if node.ident == "spacer" {
            count += 1;
        }

        Ok(ControlFlow::Continue(()))
    })?;

    if count == 0 {
        return Ok(final_size);
    }

    match axis {
        Axis::Horizontal => {
            constraints.div_assign_max_width(count);
            constraints.min_width = constraints.max_width();
        }
        Axis::Vertical => {
            constraints.div_assign_max_height(count);
            constraints.min_height = constraints.max_height();
        }
    };

    nodes.each(ctx, |ctx, node, children| {
        if node.ident != "spacer" {
            return Ok(ControlFlow::Continue(()));
        }

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
