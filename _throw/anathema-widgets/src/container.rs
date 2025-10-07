use anathema_geometry::{LocalPos, Pos, Region, Size};
use anathema_value_resolver::AttributeStorage;

use crate::error::Result;
use crate::layout::{Constraints, LayoutCtx, PositionCtx, PositionFilter, Viewport};
use crate::nodes::element::Layout;
use crate::paint::{Glyphs, PaintCtx, Unsized};
use crate::widget::{AnyWidget, ForEach};
use crate::{LayoutForEach, PaintChildren, Style, WidgetId};

#[derive(Debug, PartialEq)]
pub struct Cache {
    pub(super) size: Size,
    pub(super) pos: Option<Pos>,
    constraints: Constraints,
    pub(super) child_count: usize,
    valid: bool,
}

impl Cache {
    pub(crate) const ZERO: Self = Self {
        size: Size::ZERO,
        pos: None,
        // Constraints are `None` for the root node
        constraints: Constraints::ZERO,
        child_count: 0,
        valid: false,
    };

    const fn new(size: Size, constraints: Constraints) -> Self {
        Self {
            size,
            pos: None,
            constraints,
            child_count: 0,
            valid: true,
        }
    }

    // Get the size if the constraints are matching, but only
    // if the size is not max, as that would be an invalid cache
    pub(super) fn size(&self) -> Option<Size> {
        self.valid.then_some(self.size)
    }

    fn changed(&mut self, mut cache: Cache) -> bool {
        let changed = self.size != cache.size;
        cache.child_count = self.child_count;
        *self = cache;
        changed
    }

    pub fn constraints(&self) -> Constraints {
        self.constraints
    }
}

/// Wraps a widget and retain some geometry for the widget
#[derive(Debug)]
pub(crate) struct Container {
    pub inner: Box<dyn AnyWidget>,
    pub id: WidgetId,
    pub inner_bounds: Region,
    pub(super) cache: Cache,
}

impl Container {
    pub(crate) fn layout<'bp>(
        &mut self,
        children: LayoutForEach<'_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Result<Layout> {
        let size = self.inner.any_layout(children, constraints, self.id, ctx)?;
        let cache = Cache::new(size, constraints);

        let changed = self.cache.changed(cache);

        // Floating widgets always report a zero size
        // as they should not affect their parents
        if self.inner.any_floats() {
            return Ok(Layout::Unchanged(Size::ZERO));
        }

        // If the layout is changed but the widget is floating it will
        // have no impact on the parent widget
        let layout = match changed {
            true => match self.inner.any_floats() {
                false => Layout::Changed(self.cache.size),
                true => Layout::Floating(self.cache.size),
            },
            false => Layout::Unchanged(self.cache.size),
        };

        Ok(layout)
    }

    pub(crate) fn position<'bp>(
        &mut self,
        children: ForEach<'_, 'bp, PositionFilter>,
        pos: Pos,
        attribute_storage: &AttributeStorage<'bp>,
        viewport: Viewport,
    ) {
        if self.cache.size().is_none() {
            return;
        }

        let pos = *self.cache.pos.get_or_insert(pos);

        let ctx = PositionCtx {
            inner_size: self.cache.size,
            pos,
            viewport,
        };
        self.inner.any_position(children, self.id, attribute_storage, ctx);
        self.inner_bounds = self.inner.any_inner_bounds(pos, self.cache.size);
    }

    pub(crate) fn paint<'bp>(
        &mut self,
        children: PaintChildren<'_, 'bp>,
        ctx: PaintCtx<'_, Unsized>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        if self.cache.size().is_none() {
            return;
        }

        let mut ctx = ctx.into_sized(self.cache.size, self.cache.pos.expect("only paint laid-out widgets"));
        let region = ctx.create_region();
        ctx.set_clip_region(region);

        let attributes = attribute_storage.get(self.id);

        if !self.inner.any_floats() {
            let style = Style::from_cell_attribs(attributes);
            // Apply all attributes to the widget
            // as long as it's **not** a floating widget.
            for y in 0..self.cache.size.height {
                for x in 0..self.cache.size.width {
                    let pos = LocalPos::new(x, y);
                    ctx.set_style(style, pos);
                }
            }

            if let Some(fill) = attributes.get("fill") {
                for y in 0..ctx.local_size.height {
                    let mut used_width = 0;
                    while used_width < ctx.local_size.width {
                        let pos = LocalPos::new(used_width, y);

                        fill.strings(|s| {
                            let glyphs = Glyphs::new(s);
                            let Some(p) = ctx.place_glyphs(glyphs, pos) else {
                                return false;
                            };
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
