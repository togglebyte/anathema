use anathema_render::Size;

use super::many::Many;
use super::{expand, spacers, Constraints, Layout, Padding};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::{Error, Result};
use crate::gen::generator::Generator;
use crate::{Axis, Expand, Spacer, WidgetContainer, Direction};

pub struct Horizontal(Many);

impl Horizontal {
    pub fn new(direction: Direction) -> Self {
        let many = Many::new(direction, Axis::Horizontal, 0, false);
        Self(many)
    }
}

impl Layout for Horizontal {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        self.0.layout(ctx, size)
        // let mut used_width = 0;
        // let mut height = 0;

        // let constraints = ctx.padded_constraints();
        // let max_width = constraints.max_width;

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

        //     // Ignore expanded widgets
        //     if widget.kind() == Expand::KIND {
        //         continue;
        //     }

        //     let constraints = Constraints::new(max_width - used_width, constraints.max_height);

        //     let size = match widget.layout(constraints, &values, ctx.lookup) {
        //         Ok(s) => s,
        //         Err(Error::InsufficientSpaceAvailble) => break,
        //         err @ Err(_) => err?,
        //     };

        //     height = height.max(size.height);
        //     used_width = (used_width + size.width).min(max_width);

        //     if used_width >= max_width {
        //         break;
        //     }
        // }

        // if !ctx.constraints.is_width_unbounded() {
        //     ctx.constraints.max_width -= used_width;

        //     let expanded_size = expand::layout(ctx, Axis::Horizontal)?;
        //     height = height.max(expanded_size.height);
        //     used_width += expanded_size.width;

        //     ctx.constraints.max_height = height;
        //     let spacer_size = spacers::layout(ctx, Axis::Horizontal)?;
        //     used_width += spacer_size.width;
        // }

        // height = height.max(ctx.constraints.min_height) + ctx.padding.top + ctx.padding.bottom;
        // height = height.min(ctx.constraints.max_height);
        // size.height = size.height.max(height);

        // used_width = used_width.max(ctx.constraints.min_width);

        // size.width = size.width.max(used_width);

        // Ok(())
    }
}
