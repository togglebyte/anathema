use std::ops::ControlFlow;
use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{AsNodePath, Node, TreeValues};
use anathema_strings::HStrings;
use anathema_value_resolver::{AttributeStorage, Scope};
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionFilter, Viewport};
use anathema_widgets::paint::PaintFilter;
use anathema_widgets::tree::WidgetPositionFilter;
use anathema_widgets::{
    DirtyWidgets, Element, FloatingWidgets, ForEach, GlyphMap, LayoutForEach, PaintChildren, PositionChildren,
    WidgetContainer, WidgetGenerator, WidgetKind, WidgetTree,
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
        strings: &HStrings<'bp>,
        ignore_floats: bool,
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
        // let tree = self.tree.view_mut();
        // // Floating widgets
        // for widget_id in ctx.floating_widgets.iter() {
        //     // Find the parent widget and get the position
        //     // If no parent element is found assume Pos::ZERO
        //     let mut parent = tree.path_ref(*widget_id).parent();
        //     let (pos, constraints) = loop {
        //         match parent {
        //             None => break (Pos::ZERO, self.constraints),
        //             Some(p) => match tree.get_ref_by_path(p) {
        //                 Some(WidgetContainer {
        //                     kind: WidgetKind::Element(el),
        //                     ..
        //                 }) => {
        //                     let bounds = el.inner_bounds();
        //                     break (bounds.from, Constraints::from(bounds));
        //                 }
        //                 _ => parent = p.parent(),
        //             },
        //         }
        //     };

        //     let scope = Scope::root();
        //     let mut for_each = LayoutForEach::new(tree, &scope);
        //     for_each.each(ctx, |ctx, widget, children| {
        //         widget.layout(children, constraints, ctx);
        //         ControlFlow::Break(())
        //     });

        //     // tree.with_nodes_and_values(*widget_id, |widget, children, values| {
        //     //     let WidgetKind::Element(el) = &mut widget.kind else {
        //     //         unreachable!("this is always a floating widget")
        //     //     };

        //     //     //         layout_widget(el, children, values, constraints, &mut layout_ctx, true);

        //     //     //         // Position
        //     //     //         position_widget(pos, el, children, values, self.attribute_storage, true, self.viewport);

        //     //     //         // Paint
        //     //     //         self.backend
        //     //     //             .paint(self.glyph_map, el, children, values, self.attribute_storage, true);
        //     // });
        // }
    }

    pub fn run(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        // -----------------------------------------------------------------------------
        //   - Layout -
        // -----------------------------------------------------------------------------
        self.layout(ctx);

        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        self.position(ctx.attribute_storage, ctx.viewport);

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        self.paint(ctx);

        self.floating(ctx);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        if ctx.dirty_widgets.is_empty() && !ctx.force_layout {
            return;
        }

        let scope = Scope::root();
        let mut for_each = LayoutForEach::new(self.tree.view_mut(), &scope);
        let constraints = self.constraints;
        for_each.each(ctx, |ctx, widget, children| {
            widget.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });
    }

    fn position(&mut self, attributes: &AttributeStorage<'bp>, viewport: Viewport) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut for_each = PositionChildren::new(self.tree.view_mut(), attributes, PositionFilter::fixed());
        for_each.each(|widget, children| {
            widget.position(children, Pos::ZERO, attributes, viewport);
            ControlFlow::Break(())
        });
    }

    fn paint(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut for_each = PaintChildren::new(self.tree.view_mut(), ctx.attribute_storage, PaintFilter::fixed());
        self.backend
            .paint(ctx.glyph_map, for_each, ctx.attribute_storage, ctx.strings, true);
    }
}
