use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Layout};
use anathema_widget_core::WidgetContainer;

use crate::Spacer;

pub struct SpacerLayout;

impl Layout for SpacerLayout {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        _children: &mut Vec<WidgetContainer>,
        size: &mut Size,
    ) -> Result<()> {
        *size = Size::new(ctx.constraints.min_width, ctx.constraints.min_height);

        Ok(())
    }
}

/// Layout spacers.
/// This is different to [`SpacerLayout`] which
/// does the layout of the children of a single [`Spacer`],
/// whereas this does the layout of multiple [`Spacer`]s.
pub fn layout(
    ctx: &mut LayoutCtx<'_, '_>,
    children: &mut Vec<WidgetContainer>,
    axis: Axis,
) -> Result<Size> {
    let mut final_size = Size::ZERO;
    let count = children.iter().filter(|c| c.kind() == Spacer::KIND).count();

    if count == 0 {
        return Ok(final_size);
    }

    let mut constraints = ctx.constraints;
    match axis {
        Axis::Horizontal => {
            constraints.max_width /= count;
            constraints.min_width = constraints.max_width;
        }
        Axis::Vertical => {
            constraints.max_height /= count;
            constraints.min_height = constraints.max_height;
        }
    };

    for spacer in children.iter_mut().filter(|c| c.kind() == Spacer::KIND) {
        let size = spacer.layout(constraints, ctx.values)?;

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
