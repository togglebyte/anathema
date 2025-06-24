use std::ops::ControlFlow;
use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_value_resolver::{AttributeStorage, Scope};
use anathema_widgets::components::events::Event;
use anathema_widgets::error::Result;
use anathema_widgets::layout::{Constraints, LayoutCtx, LayoutFilter, PositionFilter, Viewport};
use anathema_widgets::paint::PaintFilter;
use anathema_widgets::{GlyphMap, LayoutForEach, PaintChildren, PositionChildren, WidgetTreeView};

pub mod testing;
pub mod tui;

pub trait Backend {
    fn size(&self) -> Size;

    fn next_event(&mut self, timeout: Duration) -> Option<Event>;

    fn resize(&mut self, new_size: Size, glyph_map: &mut GlyphMap);

    /// Paint the widgets
    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    );

    /// Called by the runtime at the end of the frame.
    fn render(&mut self, glyph_map: &mut GlyphMap);

    /// Clear is called immediately after `render` is called.
    fn clear(&mut self);

    /// Finalizes the backend. This is called when the runtime starts.
    fn finalize(&mut self) {}
}

// TODO: rename this.
// This does layout, position and paint and should have
// a less silly name
pub struct WidgetCycle<'rt, 'bp, T> {
    backend: &'rt mut T,
    tree: WidgetTreeView<'rt, 'bp>,
    constraints: Constraints,
}

impl<'rt, 'bp, T: Backend> WidgetCycle<'rt, 'bp, T> {
    pub fn new(backend: &'rt mut T, tree: WidgetTreeView<'rt, 'bp>, constraints: Constraints) -> Self {
        Self {
            backend,
            tree,
            constraints,
        }
    }

    fn fixed(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, needs_layout: bool) -> Result<()> {
        // -----------------------------------------------------------------------------
        //   - Layout -
        // -----------------------------------------------------------------------------
        if needs_layout {
            self.layout(ctx, LayoutFilter)?;
        }

        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        self.position(ctx.attribute_storage, *ctx.viewport, PositionFilter::fixed());

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        self.paint(ctx, PaintFilter::fixed());

        Ok(())
    }

    fn floating(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) -> Result<()> {
        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        self.position(ctx.attribute_storage, *ctx.viewport, PositionFilter::floating());

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        self.paint(ctx, PaintFilter::floating());

        Ok(())
    }

    pub fn run(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, needs_layout: bool) -> Result<()> {
        self.fixed(ctx, needs_layout)?;
        self.floating(ctx)?;
        Ok(())
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, filter: LayoutFilter) -> Result<()> {
        #[cfg(feature = "profile")]
        puffin::profile_function!();
        let tree = self.tree.view();

        let scope = Scope::root();
        let mut for_each = LayoutForEach::new(tree, &scope, filter, None);
        let constraints = self.constraints;
        _ = for_each.each(ctx, |ctx, widget, children| {
            _ = widget.layout(children, constraints, ctx)?;
            Ok(ControlFlow::Break(()))
        })?;
        Ok(())
    }

    fn position(&mut self, attributes: &AttributeStorage<'bp>, viewport: Viewport, filter: PositionFilter) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut for_each = PositionChildren::new(self.tree.view(), attributes, filter);
        _ = for_each.each(|widget, children| {
            widget.position(children, Pos::ZERO, attributes, viewport);
            ControlFlow::Break(())
        });
    }

    fn paint(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, filter: PaintFilter) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let for_each = PaintChildren::new(self.tree.view(), ctx.attribute_storage, filter);
        self.backend.paint(ctx.glyph_map, for_each, ctx.attribute_storage);
    }
}
