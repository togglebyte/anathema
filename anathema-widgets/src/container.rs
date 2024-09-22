use anathema_geometry::{LocalPos, Pos, Region, Size};

use crate::layout::{Constraints, LayoutCtx, PositionCtx, Viewport};
use crate::paint::{PaintCtx, Unsized};
use crate::widget::{AnyWidget, PositionChildren, WidgetNeeds};
use crate::{AttributeStorage, LayoutChildren, PaintChildren, WidgetId};

/// Wraps a widget and retain some geometry for the widget
#[derive(Debug)]
pub struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub size: Size,
    pub pos: Pos,
    pub inner_bounds: Region,
    pub needs: WidgetNeeds,
}

impl Container {
    pub fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        if !matches!(self.needs, WidgetNeeds::Layout) {
            return self.size;
        }
        self.needs = WidgetNeeds::Position;

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
        if !matches!(self.needs, WidgetNeeds::Position) && pos == self.pos {
            return;
        }
        self.needs = WidgetNeeds::Paint;

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
        mut children: PaintChildren<'_, '_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        if !matches!(self.needs, WidgetNeeds::Paint) {
            return;
        }
        self.needs = WidgetNeeds::Nothing;

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
