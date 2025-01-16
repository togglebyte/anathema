use std::ops::ControlFlow;
use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{AsNodePath, Node, TreeValues};
use anathema_strings::HStrings;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    DirtyWidgets, Element, FloatingWidgets, ForEach, GlyphMap, LayoutForEach, PaintChildren,
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

    fn floating(&mut self) {
        // // Floating widgets
        // for widget_id in self.floating_widgets.iter() {
        //     // Find the parent widget and get the position
        //     // If no parent element is found assume Pos::ZERO
        //     let mut parent = self.tree.path_ref(*widget_id).parent();
        //     let (pos, constraints) = loop {
        //         match parent {
        //             None => break (Pos::ZERO, self.constraints),
        //             Some(p) => match self.tree.get_ref_by_path(p) {
        //                 Some(WidgetContainer { kind:WidgetKind::Element(el), .. }) => {
        //                     let bounds = el.inner_bounds();
        //                     break (bounds.from, Constraints::from(bounds));
        //                 }
        //                 _ => parent = p.parent(),
        //             },
        //         }
        //     };

        //     self.tree.with_nodes_and_values(*widget_id, |widget, children, values| {
        //         let WidgetKind::Element(el) = &mut widget.kind else { unreachable!("this is always a floating widget") };
        //         let mut layout_ctx = LayoutCtx::new(
        //             self.attribute_storage,
        //             self.dirty_widgets,
        //             &self.viewport,
        //             self.glyph_map,
        //             self.force_layout,
        //         );

        //         layout_widget(el, children, values, constraints, &mut layout_ctx, true);

        //         // Position
        //         position_widget(pos, el, children, values, self.attribute_storage, true, self.viewport);

        //         // Paint
        //         self.backend
        //             .paint(self.glyph_map, el, children, values, self.attribute_storage, true);
        //     });
        // }
    }

    pub fn run(&mut self, ctx: &mut LayoutCtx<'_, 'bp>) {
        // -----------------------------------------------------------------------------
        //   - Layout -
        // -----------------------------------------------------------------------------
        let mut for_each = LayoutForEach::new(self.tree.view_mut());
        let constraints = self.constraints;
        for_each.each(ctx, |ctx, widget, children| {
            widget.layout(children, constraints, ctx);
            ControlFlow::Break(())
        });

        // -----------------------------------------------------------------------------
        //   - Position -
        // -----------------------------------------------------------------------------
        let mut for_each = PositionChildren::new(self.tree.view_mut());
        for_each.each(|widget, children| {
            widget.position(children, Pos::ZERO, ctx.attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });

        // -----------------------------------------------------------------------------
        //   - Paint -
        // -----------------------------------------------------------------------------
        let mut for_each = PaintChildren::new(self.tree.view_mut());
        self.backend
            .paint(ctx.glyph_map, for_each, ctx.attribute_storage, ctx.strings, true);

        self.floating();
    }
}
