use std::ops::ControlFlow;

use anathema_geometry::{LocalPos, Pos, Region, Size};
use anathema_templates::blueprints::Blueprint;

use crate::layout::{Constraints, LayoutCtx, PositionCtx, PositionFilter, Viewport};
use crate::paint::{Glyphs, PaintCtx, Unsized};
use crate::widget::{AnyWidget, ForEach, PositionChildren};
use crate::{AttributeStorage, LayoutForEach, LayoutChildren, PaintChildren, WidgetId};

#[derive(Debug, PartialEq)]
pub struct Cache {
    pub(super) size: Size,
    constraints: Constraints,
}

impl Cache {
    pub(crate) const ZERO: Self = Self::new(Size::ZERO, Constraints::ZERO);

    const fn new(size: Size, constraints: Constraints) -> Self {
        Self { size, constraints }
    }
}

/// Wraps a widget and retain some geometry for the widget
#[derive(Debug)]
pub(crate) struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub pos: Pos,
    pub inner_bounds: Region,
    pub cache: Cache,
}

impl Container {
    pub(crate) fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        // NOTE: The layout is possibly skipped in the Element::layout call

        let size = self.inner.any_layout(children, constraints, self.id, ctx);
        let cache = Cache::new(size, constraints);

        if cache != self.cache {
            // If this was the target node to layout and nothing has changed,
            // then there is no reason to continue the layout.
            ctx.force_layout = true;
            self.cache = cache;
        }

        // If the size does not match the previous size, or the constraints are
        // different than last frame, then this needs to layout everything.

        // Floating widgets always report a zero size
        // as they should not affect their parents
        match self.inner.any_floats() {
            true => Size::ZERO,
            false => self.cache.size,
        }
    }

    pub(crate) fn position<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        self.pos = pos;
        let ctx = PositionCtx {
            inner_size: self.cache.size,
            pos,
            viewport,
        };
        self.inner.any_position(children, self.id, attribute_storage, ctx);
        self.inner_bounds = self.inner.any_inner_bounds(self.pos, self.cache.size);
    }

    pub(crate) fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        // if !matches!(self.needs, WidgetNeeds::Paint) {
        //     return;
        // }

        let mut ctx = ctx.into_sized(self.cache.size, self.pos);
        let region = ctx.create_region();
        ctx.set_clip_region(region);

        let attributes = attribute_storage.get(self.id);

        let size = self.cache.size;

        // Apply all attributes
        for y in 0..self.cache.size.height as u16 {
            for x in 0..self.cache.size.width as u16 {
                let pos = LocalPos::new(x, y);
                ctx.set_attributes(attributes, pos);
            }
        }

        if let Some(fill) = attributes.get_val("fill") {
            for y in 0..ctx.local_size.height as u16 {
                let mut used_width = 0;
                loop {
                    let pos = LocalPos::new(used_width, y);
                    let controlflow = fill.str_iter(|s| {
                        let glyphs = Glyphs::new(s);
                        let Some(p) = ctx.place_glyphs(glyphs, pos) else {
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

        self.inner.any_paint(children, self.id, attribute_storage, ctx)
    }
}
