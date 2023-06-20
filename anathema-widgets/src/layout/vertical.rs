use anathema_render::Size;

use super::many::Many;
use super::{Layout};
use crate::contexts::{LayoutCtx};
use crate::error::{Result};

use crate::{Axis, Direction};

#[derive(Debug)]
pub struct Vertical(Many);

impl Vertical {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Vertical, 0, false);
        Self(many)
    }
}

impl Layout for Vertical {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        self.0.layout(ctx, size)
        // let mut used_height = 0;
        // let mut width = 0;

        // let constraints = ctx.padded_constraints();
        // let max_height = constraints.max_height;

        // let mut values = ctx.values.next();
        // let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);

        // while let Some(mut widget) = gen.next(&mut values).transpose()? {
        //     let index = ctx.children.len();
        //     ctx.children.push(widget);
        //     let widget = &mut ctx.children[index];

        //     // Ignore spacers
        //     if widget.kind() == Spacer::KIND {
        //         continue;
        //     }

        //     // Ignore expanded
        //     if widget.kind() == Expand::KIND {
        //         continue;
        //     }

        //     let constraints = Constraints::new(constraints.max_width, max_height - used_height);

        //     let size = match widget.layout(constraints, &values, ctx.lookup) {
        //         Ok(s) => s,
        //         Err(Error::InsufficientSpaceAvailble) => break,
        //         err @ Err(_) => err?,
        //     };

        //     width = width.max(size.width);
        //     used_height = (used_height + size.height).min(max_height);

        //     if used_height >= max_height {
        //         break;
        //     }
        // }

        // if !ctx.constraints.is_height_unbounded() {
        //     ctx.constraints.max_height -= used_height;

        //     let expanded_size = expand::layout(ctx, Axis::Vertical)?;
        //     width = width.max(expanded_size.width);
        //     used_height += expanded_size.height;

        //     ctx.constraints.max_width = width;
        //     let spacer_size = spacers::layout(ctx, Axis::Vertical)?;
        //     used_height += spacer_size.height;
        // }

        // width = width.max(ctx.constraints.min_width) + ctx.padding.left + ctx.padding.right;
        // width = width.min(ctx.constraints.max_width);
        // size.width = size.width.max(width);

        // used_height = used_height.max(ctx.constraints.min_height);

        // size.height = size.height.max(used_height);

        // Ok(())
    }
}
