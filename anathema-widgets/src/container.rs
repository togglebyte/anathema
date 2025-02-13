use std::ops::ControlFlow;

use anathema_geometry::{LocalPos, Pos, Region, Size};
use anathema_templates::blueprints::Blueprint;
use anathema_value_resolver::AttributeStorage;

use crate::layout::{Constraints, LayoutCtx, PositionCtx, PositionFilter, Viewport};
use crate::nodes::element::Layout;
use crate::paint::{Glyphs, PaintCtx, Unsized};
use crate::widget::{AnyWidget, ForEach, PositionChildren};
use crate::{LayoutChildren, LayoutForEach, PaintChildren, WidgetId};

#[derive(Debug, PartialEq)]
pub struct Cache {
    pub(super) size: Size,
    constraints: Option<Constraints>,
    valid: bool,
}

impl Cache {
    pub(crate) const ZERO: Self = Self {
        size: Size::ZERO,
        constraints: None,
        valid: false,
    };

    const fn new(size: Size, constraints: Constraints) -> Self {
        Self {
            size,
            constraints: Some(constraints),
            valid: true,
        }
    }

    // Get the size if the constraints are matching, but only
    // if the size is not max, as that would be an invalid cache
    pub(super) fn size(&self) -> Option<Size> {
        self.valid.then(|| self.size)
    }

    pub(super) fn invalidate(&mut self) {
        self.valid = false;
    }

    fn changed(&mut self, cache: Cache) -> bool {
        let changed = self.size != cache.size;
        *self = cache;
        changed
    }

    pub(crate) fn constraints(&self) -> Option<Constraints> {
        self.constraints
    }
}

/// Wraps a widget and retain some geometry for the widget
#[derive(Debug)]
pub(crate) struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub pos: Pos,
    pub inner_bounds: Region,
    pub(super) cache: Cache,
}

impl Container {
    pub(crate) fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Layout {
        // NOTE: The layout is possibly skipped in the Element::layout call

        let size = self.inner.any_layout(children, constraints, self.id, ctx);
        let cache = Cache::new(size, constraints);

        let changed = self.cache.changed(cache);
        match changed {
            true => Layout::Changed(self.cache.size),
            false => Layout::Unchanged(self.cache.size),
        }

        // Floating widgets always report a zero size
        // as they should not affect their parents
        // match self.inner.any_floats() {
        //     true => Layout::Unchanged(Size::ZERO),
        //     false => self.cache.size,
        // }
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
        let mut ctx = ctx.into_sized(self.cache.size, self.pos);
        let region = ctx.create_region();
        ctx.set_clip_region(region);

        let attributes = attribute_storage.get(self.id);

        let size = self.cache.size;

        // Apply all attributes to the widget
        // as long as it's **not** a floating widget.
        if !self.inner.any_floats() {
            for y in 0..self.cache.size.height as u16 {
                for x in 0..self.cache.size.width as u16 {
                    let pos = LocalPos::new(x, y);
                    ctx.set_attributes(attributes, pos);
                }
            }

            if let Some(fill) = attributes.get("fill") {
                for y in 0..ctx.local_size.height as u16 {
                    let mut used_width = 0;
                    while used_width < ctx.local_size.width as u16 {
                        let pos = LocalPos::new(used_width, y);

                        fill.strings(|s| {
                            let glyphs = Glyphs::new(s);
                            let Some(p) = ctx.place_glyphs(glyphs, pos) else {
                                return false;
                            };
                            let width = ctx.local_size.width;
                            used_width = p.x;
                            true
                        });
                    }
                }
            }
        }

        self.inner.any_paint(children, self.id, attribute_storage, ctx)
    }
}
