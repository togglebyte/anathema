use anathema_render::Size;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::WidgetContainer;

pub struct Stacked;

impl Layout for Stacked {
    fn layout(
        &mut self,
        layout: &mut LayoutCtx,
        children: &mut anathema_widget_core::Nodes,
        data: anathema_values::Context<'_, '_>,
    ) -> Result<Size> {
        let mut width = 0;
        let mut height = 0;

        let constraints = layout.padded_constraints();

        children.for_each(data.state, data.scope, layout, |widget, children, data| {
            let size = match widget.layout(children, constraints, data) {
                Ok(s) => s,
                err @ Err(_) => err?,
            };

            width = width.max(size.width);
            height = height.max(size.height);

            Ok(size)
        });

        Ok(Size { width, height })
    }
}
