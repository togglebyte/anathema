use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{WidgetContainer, Nodes}; 

pub struct Single;

impl Layout for Single {
    fn layout<'widget, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut Nodes,
        store: &StoreRef<'_>,
        size: &mut Size,
    ) -> Result<()> {
        let constraints = ctx.padded_constraints();

        if let Some((widget, children)) = children.next(store).transpose()? {
            *size = match widget.layout(children, constraints, store) {
                Ok(s) => s,
                Err(Error::InsufficientSpaceAvailble) => return Ok(()),
                err @ Err(_) => err?,
            };
        }

        Ok(())
    }
}
