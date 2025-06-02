use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::LayoutForEach;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx};

use crate::border::BorderSize;

pub struct BorderLayout {
    pub min_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_width: Option<u16>,
    pub max_height: Option<u16>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub border_size: BorderSize,
}

impl BorderLayout {
    pub(crate) fn layout<'bp>(
        &mut self,
        mut children: LayoutForEach<'_, 'bp>,
        mut constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Size> {
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

        _ = children.each(ctx, |ctx, child, children| {
            // Shrink the constraint for the child to fit inside the border

            // border [min-width: 10]
            //     border [min-width: 18]
            //         ...

            // border [max-width: 8]
            //     border [min-width: 10]
            //         ...

            child_constraints.sub_max_width((border_size.left + border_size.right) as u16);
            child_constraints.sub_max_height((border_size.top + border_size.bottom) as u16);
            let mut child_size = Size::from(child.layout(children, child_constraints, ctx)?);
            child_size += border_size.as_size();
            size.width = child_size.width.max(size.width);
            size.height = child_size.height.max(size.height);

            Ok(ControlFlow::Break(()))
        })?;

        size.width = constraints.min_width.max(size.width).min(constraints.max_width());
        size.height = constraints.min_height.max(size.height).min(constraints.max_height());

        Ok(size)
    }
}
