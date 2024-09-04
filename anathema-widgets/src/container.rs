use anathema_geometry::{LocalPos, Pos, Size};

use crate::layout::text::StringSession;
use crate::layout::{Constraints, LayoutCtx, PositionCtx, Viewport};
use crate::paint::{PaintCtx, Unsized};
use crate::widget::{AnyWidget, PositionChildren};
use crate::{AttributeStorage, LayoutChildren, PaintChildren, WidgetId};

#[derive(Debug)]
pub struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub size: Size,
    pub pos: Pos,
}

impl Container {
    pub fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, '_, 'bp>,
    ) -> Size {
        self.size = self.inner.any_layout(children, constraints, self.id, ctx);
        // Floating widgets always report a zero size
        // as they should not affect their parents
        match self.inner.any_floats() {
            true => Size::ZERO,
            false => self.size,
        }
    }

    pub fn position<'bp>(
        &mut self,
        children: PositionChildren<'_, '_, 'bp>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        self.pos = pos;
        let ctx = PositionCtx {
            inner_size: self.size,
            pos,
            viewport,
        };
        self.inner.any_position(children, self.id, attribute_storage, ctx);
    }

    pub fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        text: &mut StringSession<'_>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        let mut ctx = ctx.into_sized(self.size, self.pos);
        let region = ctx.create_region();
        ctx.set_clip_region(region);

        let attrs = attribute_storage.get(self.id);

        // Apply all attributes
        for y in 0..self.size.height as u16 {
            for x in 0..self.size.width as u16 {
                let pos = LocalPos::new(x, y);
                ctx.set_attributes(attrs, pos);
            }
        }

        self.inner.any_paint(children, self.id, attribute_storage, ctx, text)
    }
}
