use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{Generator, WidgetContainer};

pub struct Single;

impl Layout for Single {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'parent>,
        children: &mut Vec<WidgetContainer>,
        size: &mut Size,
    ) -> Result<()> {
        let constraints = ctx.padded_constraints();
        let mut values = ctx.values.next();
        let mut gen = Generator::new(&ctx.templates, &mut values);

        if let Some(widget) = gen.next(&mut values).transpose()? {
            children.push(widget);
            *size = match children[0].layout(constraints, &values) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => return Ok(()),
                err @ Err(_) => err?,
            };
        }

        Ok(())
    }
}
