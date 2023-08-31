use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::{Constraints, Layout};
use anathema_widget_core::{WidgetContainer, Nodes};

pub struct BorderLayout {
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub border_size: Size,
}

impl Layout for BorderLayout {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Nodes,
        size: &mut Size,
    ) -> Result<()> {
        panic!()
        // // If there is a min width / height, make sure the minimum constraints
        // // are matching these
        // if let Some(min_width) = self.min_width {
        //     ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        // }

        // if let Some(min_height) = self.min_height {
        //     ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        // }

        // // If there is a width / height then make the constraints tight
        // // around the size. This will modify the size to fit within the
        // // constraints first.
        // if let Some(width) = self.width {
        //     ctx.constraints.make_width_tight(width);
        // }

        // if let Some(height) = self.height {
        //     ctx.constraints.make_height_tight(height);
        // }

        // if ctx.constraints == Constraints::ZERO {
        //     return Ok(());
        // }

        // let border_size = self.border_size;

        // *size = match children.next(store).transpose()? {
        //     Some((mut widget, children)) => {
        //         let mut constraints = ctx.padded_constraints();

        //         // Shrink the constraint for the child to fit inside the border
        //         constraints.max_width = match constraints.max_width.checked_sub(border_size.width) {
        //             Some(w) => w,
        //             None => return Err(Error::InsufficientSpaceAvailble),
        //         };

        //         constraints.max_height =
        //             match constraints.max_height.checked_sub(border_size.height) {
        //                 Some(h) => h,
        //                 None => return Err(Error::InsufficientSpaceAvailble),
        //             };

        //         if constraints.min_width > constraints.max_width {
        //             constraints.min_width = constraints.max_width;
        //         }

        //         if constraints.min_height > constraints.max_height {
        //             constraints.min_height = constraints.max_height;
        //         }

        //         if constraints.max_width == 0 || constraints.max_height == 0 {
        //             return Err(Error::InsufficientSpaceAvailble);
        //         }

        //         let size = match widget.layout(children, constraints, store) {
        //             Ok(s) => s,
        //             Err(Error::InsufficientSpaceAvailble) => return Ok(()),
        //             err @ Err(_) => err?,
        //         };
        //         let mut size = size + border_size + ctx.padding_size();

        //         if let Some(min_width) = self.min_width {
        //             size.width = size.width.max(min_width);
        //         }

        //         if let Some(min_height) = self.min_height {
        //             size.height = size.height.max(min_height);
        //         }

        //         if ctx.constraints.is_width_tight() {
        //             size.width = ctx.constraints.max_width;
        //         }

        //         if ctx.constraints.is_height_tight() {
        //             size.height = ctx.constraints.max_height;
        //         }

        //         Size {
        //             width: size.width.min(ctx.constraints.max_width),
        //             height: size.height.min(ctx.constraints.max_height),
        //         }
        //     }
        //     None => {
        //         let mut size = Size::new(ctx.constraints.min_width, ctx.constraints.min_height);
        //         if ctx.constraints.is_width_tight() {
        //             size.width = ctx.constraints.max_width;
        //         }
        //         if ctx.constraints.is_height_tight() {
        //             size.height = ctx.constraints.max_height;
        //         }
        //         size
        //     }
        // };

        // Ok(())
    }
}
