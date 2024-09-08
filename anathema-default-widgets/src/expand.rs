use std::ops::ControlFlow;

use anathema_geometry::{LocalPos, Size};
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{AttributeStorage, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId};

use crate::layout::{single_layout, Axis};

#[derive(Debug, Default)]
pub struct Expand;

impl Widget for Expand {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let mut size = single_layout(children, constraints, ctx);

        let attributes = ctx.attribs.get(id);
        match attributes.get("axis") {
            Some(Axis::Horizontal) => size.width = constraints.max_width(),
            Some(Axis::Vertical) => size.height = constraints.max_height(),
            None => {
                size.width = constraints.max_width();
                size.height = constraints.max_height();
            }
        }

        size
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        _attributes: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        ctx: PositionCtx,
    ) {
        children.for_each(|node, children| {
            node.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, '_, 'bp>,
        id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        let attributes = attribute_storage.get(id);
        if let Some(fill) = attributes.get_val("fill") {
            for y in 0..ctx.local_size.height as u16 {
                let mut used_width = 0;
                loop {
                    let pos = LocalPos::new(used_width, y);
                    let controlflow = fill.str_iter(|s| {
                        let Some(p) = ctx.place_glyphs(s, pos) else {
                            return ControlFlow::Break(());
                        };
                        used_width += p.x - used_width;
                        match used_width >= ctx.local_size.width as u16 {
                            true => ControlFlow::Break(()),
                            false => ControlFlow::Continue(()),
                        }
                    });

                    if let ControlFlow::Break(()) = controlflow {
                        break;
                    }
                }
            }
        }

        children.for_each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Break(())
        });
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn expand() {}
}
