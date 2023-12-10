use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::nodes::Nodes;
use anathema_widget_core::layout::{Constraints, Layout};
use anathema_widget_core::{LayoutNodes, WidgetContainer};

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
        });

        Ok(size)
    }

    // fn layout<'widget, 'parent>(
    //     &mut self,
    //     ctx: &mut LayoutCtx,
    //     children: &mut Nodes,
    //     data: Context<'_, '_>,
    // ) -> Result<()> {
    //     let constraints = ctx.padded_constraints();

    //     if let Some(size) = children.next(data.state, data.scope, ctx).transpose()? {
    //         self.0 = size;
    //         // TODO do we need to deal with insufficient space here?
    //     //     *size = match widget.layout(children, constraints, store) {
    //     //         Ok(s) => s,
    //     //         Err(Error::InsufficientSpaceAvailble) => return Ok(()),
    //     //         err @ Err(_) => err?,
    //     //     };
    //     }

    //     Ok(())
    // }

    // fn finalize(&mut self, nodes: &mut Nodes) -> Size {
    //     self.0
    // }
}
