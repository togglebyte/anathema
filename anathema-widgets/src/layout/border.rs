use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::{Constraints, Layout};
use anathema_widget_core::{Nodes, WidgetContainer};

pub struct BorderLayout {
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub border_size: Size,
}

impl Layout for BorderLayout {
    fn layout(
        &mut self,
        children: &mut Nodes<'_>,
        layout: &LayoutCtx,
        data: &Context<'_, '_>,
    ) -> Result<Size> {
        // If there is a min width / height, make sure the minimum constraints
        // are matching these
        if let Some(min_width) = self.min_width {
            layout.constraints.min_width = layout.constraints.min_width.max(min_width);
        }

        if let Some(min_height) = self.min_height {
            layout.constraints.min_height = layout.constraints.min_height.max(min_height);
        }

        // If there is a width / height then make the constraints tight
        // around the size. This will modify the size to fit within the
        // constraints first.
        if let Some(width) = self.width {
            layout.constraints.make_width_tight(width);
        }

        if let Some(height) = self.height {
            layout.constraints.make_height_tight(height);
        }

        if layout.constraints == Constraints::ZERO {
            return Ok(Size::ZERO);
        }

        let border_size = self.border_size;

        let mut constraints = layout.padded_constraints();
        let padding_size = layout.padding_size();

        let is_height_tight = layout.constraints.is_height_tight();
        let is_width_tight = layout.constraints.is_width_tight();

        let mut size = Size::ZERO;
        children.for_each(data, layout, |widget: &mut WidgetContainer, children: &mut Nodes<'_>, data: &Context<'_, '_>| {

            // Shrink the constraint for the child to fit inside the border
            constraints.max_width = match constraints.max_width.checked_sub(border_size.width) {
                Some(w) => w,
                None => return Err(Error::InsufficientSpaceAvailble),
            };

            constraints.max_height = match constraints.max_height.checked_sub(border_size.height) {
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

            let inner_size = match widget.layout(children, constraints, data) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => return Ok(()),
                err @ Err(_) => err?,
            };

            size = inner_size + border_size + padding_size;

            if let Some(min_width) = self.min_width {
                size.width = size.width.max(min_width);
            }

            if let Some(min_height) = self.min_height {
                size.height = size.height.max(min_height);
            }

            if is_width_tight {
                size.width = constraints.max_width;
            }

            if is_height_tight {
                size.height = constraints.max_height;
            }

            // TODO: is this really needed? This is the cause for a bug
            // let size = Size {
            //     width: size.width.min(constraints.max_width),
            //     height: size.height.min(constraints.max_height),
            // };

            Ok(())
        });

        match size {
            Size::ZERO => {
                let mut size =
                    Size::new(layout.constraints.min_width, layout.constraints.min_height);
                if is_width_tight {
                    size.width = layout.constraints.max_width;
                }
                if is_height_tight {
                    size.height = layout.constraints.max_height;
                }
                Ok(size)
            }
            _ => Ok(size),
        }
    }
}
