use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{WidgetContainer, Nodes, StoreRef};

pub struct Single;

impl Layout for Single {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Nodes,
        bucket: &StoreRef<'_>,
        size: &mut Size,
    ) -> Result<()> {
        let constraints = ctx.padded_constraints();

        if let Some((widget, children)) = children.next(bucket).transpose()? {
            *size = match widget.layout(children, constraints, bucket) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => return Ok(()),
                err @ Err(_) => err?,
            };
        }

        Ok(())
    }
}
