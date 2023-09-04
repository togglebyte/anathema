use anathema_render::Size;
use anathema_values::Context;
use anathema_widget_core::contexts::LayoutCtx;
use anathema_widget_core::error::{Error, Result};
use anathema_widget_core::generator::Nodes;
use anathema_widget_core::layout::Layout;
use anathema_widget_core::WidgetContainer;

pub struct Single;

impl Layout for Single {
    fn layout(&mut self, ctx: &mut LayoutCtx, children: &mut Nodes, data: Context<'_, '_>) -> Result<Size> {
         let constraints = ctx.padded_constraints();

         let size = children.next(data.state, data.scope, ctx, &mut |widget, children, data| {
             widget.layout(children, constraints, data)
         });

         match size {
             Some(Err(Error::InsufficientSpaceAvailble)) => return Ok(Size::ZERO),
             Some(size) => size,
             None => Ok(Size::ZERO),
         }

         // TODO do we need to deal with insufficient space here?
         //     *size = match widget.layout(children, constraints, store) {
         //         Ok(s) => s,
         //         Err(Error::InsufficientSpaceAvailble) => return Ok(()),
         //         err @ Err(_) => err?,
         //     };
    }

    fn finalize(&mut self, nodes: &mut Nodes) -> Size {
        todo!()
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
