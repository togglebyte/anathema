use anathema_render::Size;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Layout;
use anathema_widget_core::LayoutNodes;

pub struct Single;

impl Layout for Single {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size> {
        let mut constraints = nodes.constraints;
        constraints.apply_padding(nodes.padding);
        let mut size = Size::ZERO;

        nodes.next(|mut node| {
            size = node.layout(constraints)?;
            Ok(())
        })?;

        Ok(size)
    }
}
