use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::layout::Layout;
use anathema_widget_core::{LayoutNodes, Nodes, WidgetContainer};

pub struct Stacked;

impl Layout for Stacked {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        let mut width = 0;
        let mut height = 0;

        let mut constraints = nodes.constraints;
        constraints.apply_padding(nodes.padding);

        nodes.for_each(|mut node| {
            let widget_size = match node.layout(constraints) {
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
