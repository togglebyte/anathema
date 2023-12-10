use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::{Constraints, Layout};
use anathema_widget_core::{Nodes, WidgetContainer, LayoutNodes};

pub struct BorderLayout {
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub border_size: Size,
}

impl Layout for BorderLayout {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        // If there is a min width / height, make sure the minimum constraints
        // are matching these
        let mut constraints = nodes.constraints;

        if let Some(min_width) = self.min_width {
            constraints.min_width = constraints.min_width.max(min_width);
        }

        if let Some(min_height) = self.min_height {
            constraints.min_height = constraints.min_height.max(min_height);
        }

        // If there is a width / height then make the constraints tight
        // around the size. This will modify the size to fit within the
        // constraints first.
        if let Some(width) = self.width {
            constraints.make_width_tight(width);
        }

        if let Some(height) = self.height {
            constraints.make_height_tight(height);
        }

        if constraints == Constraints::ZERO {
            return Ok(Size::ZERO);
        }

        let border_size = self.border_size;

        constraints.apply_padding(nodes.padding);
        let padding_size = nodes.padding_size();

        let is_height_tight = constraints.is_height_tight();
        let is_width_tight = constraints.is_width_tight();

        let mut size = Size::ZERO;
        nodes.next(|mut node| {
                // Shrink the constraint for the child to fit inside the border
                constraints.max_width = match constraints.max_width.checked_sub(border_size.width) {
                    Some(w) => w,
                    None => return Err(Error::InsufficientSpaceAvailble),
                };

                constraints.max_height =
                    match constraints.max_height.checked_sub(border_size.height) {
                        Some(h) => h,
                        None => return Err(Error::InsufficientSpaceAvailble),
                    };

                if constraints.min_width > constraints.max_width {
                    constraints.min_width = constraints.max_width;
                }

                if constraints.min_height > constraints.max_height {
                    constraints.min_height = constraints.max_height;
                }

                if constraints.max_width == 0 || constraints.max_height == 0 {
                    return Err(Error::InsufficientSpaceAvailble);
                }

                let inner_size = node.layout(constraints)?;

                size = inner_size + border_size + padding_size;

                if let Some(min_width) = self.min_width {
                    size.width = size.width.max(min_width);
                }

                if let Some(min_height) = self.min_height {
                    size.height = size.height.max(min_height);
                }

                Ok(())
            },
        );

        match size {
            Size::ZERO => {
                let mut size =
                    Size::new(constraints.min_width, constraints.min_height);
                if is_width_tight {
                    size.width = constraints.max_width;
                }
                if is_height_tight {
                    size.height = constraints.max_height;
                }
                Ok(size)
            }
            _ => Ok(size),
        }
    }
}
