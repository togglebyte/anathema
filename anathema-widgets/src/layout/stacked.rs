use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{Nodes, WidgetContainer};

pub struct Stacked;

impl Layout for Stacked {
    fn layout<'e>(
        &mut self,
        children: &mut Nodes<'e>,
        layout: &LayoutCtx,
        data: &Context<'_, 'e>,
    ) -> Result<Size> {
        let mut width = 0;
        let mut height = 0;

        let constraints = layout.padded_constraints();

        children.for_each(data, layout, |widget, children, data| {
            let widget_size = match widget.layout(children, constraints, data) {
                Ok(s) => s,
                err @ Err(_) => err?,
            };

            width = width.max(widget_size.width);
            height = height.max(widget_size.height);

            Ok(())
        });

        Ok(Size { width, height })
    }
}
