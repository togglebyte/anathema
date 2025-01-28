use std::ops::ControlFlow;
use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{AsNodePath, Node, TreeValues};
use anathema_strings::HStrings;
use anathema_value_resolver::{AttributeStorage, Scope};
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{Constraints, LayoutCtx, LayoutFilter, PositionFilter, Viewport};
use anathema_widgets::paint::PaintFilter;
use anathema_widgets::tree::WidgetPositionFilter;
use anathema_widgets::{
    awful_debug, DirtyWidgets, Element, FloatingWidgets, ForEach, GlyphMap, LayoutForEach, PaintChildren,
    PositionChildren, WidgetContainer, WidgetGenerator, WidgetKind, WidgetTree,
};

pub mod test;
pub mod tui;
pub mod tuiscroll;

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
    tree: &'rt mut WidgetTree<'bp>,
    constraints: Constraints,
}

impl<'rt, 'bp, T: Backend> WidgetCycle<'rt, 'bp, T> {
    pub fn new(backend: &'rt mut T, tree: &'rt mut WidgetTree<'bp>, constraints: Constraints) -> Self {
        Self {
            backend,
            tree,
            constraints,
        }
    }

    fn floating(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        // -----------------------------------------------------------------------------
        //   - Layout -
        // -----------------------------------------------------------------------------
        self.layout(ctx, LayoutFilter::floating());

        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        self.position(ctx.attribute_storage, *ctx.viewport, PositionFilter::floating());

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        self.paint(ctx, PaintFilter::floating());
    }

    fn fixed(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        // -----------------------------------------------------------------------------
        //   - Layout -
        // -----------------------------------------------------------------------------
        self.layout(ctx, LayoutFilter::fixed());

        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        self.position(ctx.attribute_storage, *ctx.viewport, PositionFilter::fixed());

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        self.paint(ctx, PaintFilter::fixed());
    }

    pub fn run(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        self.fixed(ctx);
        self.floating(ctx);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, filter: LayoutFilter) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        if ctx.dirty_widgets.is_empty() && !ctx.force_layout {
            return;
        }

        // TODO: this is a hack.
        // This forces the entire tree to be laid out.
        //
        // This is just intermediary until we figure out how we are going to
        // do this.
        //
        // One way would be to call layout directly on the widget and if it affects
        // the size then propagate the change outwards
        ctx.force_layout = true;

        let scope = Scope::root();
        let mut for_each = LayoutForEach::new(self.tree.view_mut(), &scope, filter, None);
        let constraints = self.constraints;
        for_each.each(ctx, |ctx, widget, children| {
            widget.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });
    }

    fn position(&mut self, attributes: &AttributeStorage<'bp>, viewport: Viewport, filter: PositionFilter) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut for_each = PositionChildren::new(self.tree.view_mut(), attributes, filter);
        for_each.each(|widget, children| {
            widget.position(children, Pos::ZERO, attributes, viewport);
            ControlFlow::Break(())
        });
    }

    fn paint(&mut self, ctx: &mut LayoutCtx<'_, 'bp>, filter: PaintFilter) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut for_each = PaintChildren::new(self.tree.view_mut(), ctx.attribute_storage, filter);
        self.backend.paint(ctx.glyph_map, for_each, ctx.attribute_storage);
    }
}
