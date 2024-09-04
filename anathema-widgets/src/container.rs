use anathema_geometry::{LocalPos, Pos, Rect, Size};

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
    pub inner_bounds: Rect,
    pub needs_layout: bool,
    pub needs_position: bool,
}

impl Container {
    pub fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        if !self.needs_layout {
            return self.size;
        }
        self.needs_layout = false;
        self.needs_position = true;

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
        if !self.needs_position && pos == self.pos {
            return;
        }
        self.needs_position = false;

        self.pos = pos;
        let ctx = PositionCtx {
            inner_size: self.size,
            pos,
            viewport,
        };
        self.inner.any_position(children, self.id, attribute_storage, ctx);
        self.inner_bounds = self.inner.any_inner_bounds(self.pos, self.size);
    }

    pub fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        if self.needs_layout || self.needs_position {
            return;
        }
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

        self.inner.any_paint(children, self.id, attribute_storage, ctx)
    }
}
