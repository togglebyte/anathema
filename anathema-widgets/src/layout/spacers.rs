use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Axis, Layout};
use anathema_widget_core::{LayoutNodes, Nodes, WidgetContainer};

use crate::Spacer;

pub struct SpacerLayout;

impl Layout for SpacerLayout {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        let size = Size::new(nodes.constraints.min_width, nodes.constraints.min_height);
        Ok(size)
    }
}

/// Layout spacers.
/// This is different to [`SpacerLayout`] which
/// does the layout of the child of a single [`Spacer`],
/// whereas this does the layout of multiple [`Spacer`]s
/// inside already evaluated children.
pub fn layout<'nodes, 'expr, 'state>(
    nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    // ctx: &LayoutCtx,
    // children: &mut Nodes<'e>,
    axis: Axis,
    // data: &Context<'_, 'e>,
) -> Result<Size> {
    let mut final_size = Size::ZERO;
    let count = nodes.filter(|widget| widget.kind() == Spacer::KIND).count();

    if count == 0 {
        return Ok(final_size);
    }

    let mut constraints = nodes.constraints;
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
    nodes.set_constraints(constraints);

    for mut spacer in nodes.filter(|widget| widget.kind() == Spacer::KIND) {
        let size = spacer.layout(constraints)?;

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
