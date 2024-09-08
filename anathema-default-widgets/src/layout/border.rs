use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx};
use anathema_widgets::LayoutChildren;

use crate::border::BorderSize;

pub struct BorderLayout {
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub border_size: BorderSize,
}

impl BorderLayout {
    pub(crate) fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        mut constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let mut size = Size::ZERO;

        if let Some(min_width) = self.min_width {
            constraints.min_width = constraints.min_width.max(min_width);
        }

        if let Some(min_height) = self.min_height {
            constraints.min_height = constraints.min_height.max(min_height);
        }

        if let Some(width) = self.max_width {
            constraints.set_max_width(width);
        }

        if let Some(height) = self.max_height {
            constraints.set_max_height(height);
        }

        // If there is a width / height then make the constraints tight
        // around the size. This will modify the size to fit within the
        // constraints first.
        if let Some(width) = self.width {
            constraints.set_max_width(width);
            size.width = width;
        }

        if let Some(height) = self.height {
            constraints.set_max_height(height);
            size.height = height;
        }

        let border_size = self.border_size;

        let mut child_constraints = constraints;

        children.for_each(|child, children| {
            //     nodes.next(|mut node, context| {

            // Shrink the constraint for the child to fit inside the border

            // border [min-width: 10]
            //     border [min-width: 18]
            //         ...

            // border [max-width: 8]
            //     border [min-width: 10]
            //         ...

            child_constraints.sub_max_width((border_size.left + border_size.right) as usize);
            child_constraints.sub_max_height((border_size.top + border_size.bottom) as usize);
            let mut child_size = child.layout(children, child_constraints, ctx);
            child_size += border_size.as_size();
            size.width = child_size.width.max(size.width);
            size.height = child_size.height.max(size.height);

            ControlFlow::Break(())
        });

        size.width = constraints.min_width.max(size.width).min(constraints.max_width());
        size.height = constraints.min_height.max(size.height).min(constraints.max_height());

        size
    }
}
