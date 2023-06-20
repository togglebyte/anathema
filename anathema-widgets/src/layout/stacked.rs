use anathema_render::Size;

use super::{Constraints, Layout, Padding};
use crate::contexts::{LayoutCtx, PositionCtx};
use crate::error::{Error, Result};
use crate::gen::generator::Generator;
use crate::{Axis, WidgetContainer};

pub struct Stacked;

impl Layout for Stacked {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()> {
        let mut width = 0;
        let mut height = 0;

        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();
        let mut gen = Generator::new(ctx.templates, ctx.lookup, &mut values);

        while let Some(mut widget) = gen.next(&mut values).transpose()? {
            let index = ctx.children.len();
            ctx.children.push(widget);
            // Ignore spacers
            // if widget.kind() == Spacer::KIND {
            //     continue;
            // }

            // // Ignore expanded
            // if widget.kind() == Expand::KIND {
            //     continue;
            // }

            let size = match ctx.children[index].layout(constraints, &values, ctx.lookup) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => break,
                err @ Err(_) => err?,
            };

            width = width.max(size.width);
            height = height.max(size.height);
        }

        size.width = size.width.max(width);
        size.height = size.height.max(height);

        Ok(())
    }
}
