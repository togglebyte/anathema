use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{Generator, WidgetContainer};

pub struct Stacked;

impl Layout for Stacked {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        children: &mut Vec<WidgetContainer>,
        size: &mut Size,
    ) -> Result<()> {
        let mut width = 0;
        let mut height = 0;

        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();
        let mut gen = Generator::new(&ctx.templates, &mut values);

        while let Some(widget) = gen.next(&mut values).transpose()? {
            let index = children.len();
            children.push(widget);
            let size = match children[index].layout(constraints, &values) {
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
