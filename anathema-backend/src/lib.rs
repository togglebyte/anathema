use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{AsNodePath, Node, TreeValues};
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{AttributeStorage, DirtyWidgets, Element, FloatingWidgets, GlyphMap, WidgetContainer, WidgetKind, WidgetTree};

pub mod test;
pub mod tui;

pub trait Backend {
    fn size(&self) -> Size;

    fn next_event(&mut self, timeout: Duration) -> Option<Event>;

    fn resize(&mut self, new_size: Size, glyph_map: &mut GlyphMap);

    /// Paint the widgets
    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetContainer<'bp>>,
        attribute_storage: &AttributeStorage<'bp>,
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
    glyph_map: &'rt mut GlyphMap,
    constraints: Constraints,
    attribute_storage: &'rt AttributeStorage<'bp>,
    dirty_widgets: &'rt DirtyWidgets,
    floating_widgets: &'rt FloatingWidgets,
    viewport: Viewport,
    force_layout: bool,
}

impl<'rt, 'bp, T: Backend> WidgetCycle<'rt, 'bp, T> {
    pub fn new(
        backend: &'rt mut T,
        tree: &'rt mut WidgetTree<'bp>,
        glyph_map: &'rt mut GlyphMap,
        constraints: Constraints,
        attribute_storage: &'rt AttributeStorage<'bp>,
        dirty_widgets: &'rt DirtyWidgets,
        floating_widgets: &'rt FloatingWidgets,
        viewport: Viewport,
        force_layout: bool,
    ) -> Self {
        Self {
            backend,
            tree,
            glyph_map,
            constraints,
            attribute_storage,
            dirty_widgets,
            floating_widgets,
            viewport,
            force_layout,
        }
    }

    fn floating(&mut self) {
        // Floating widgets
        for widget_id in self.floating_widgets.iter() {
            // Find the parent widget and get the position
            // If no parent element is found assume Pos::ZERO
            let mut parent = self.tree.path_ref(*widget_id).parent();
            let (pos, constraints) = loop {
                match parent {
                    None => break (Pos::ZERO, self.constraints),
                    Some(p) => match self.tree.get_ref_by_path(p) {
                        Some(WidgetContainer { kind:WidgetKind::Element(el), .. }) => {
                            let bounds = el.inner_bounds();
                            break (bounds.from, Constraints::from(bounds));
                        }
                        _ => parent = p.parent(),
                    },
                }
            };

            self.tree.with_nodes_and_values(*widget_id, |widget, children, values| {
                let WidgetKind::Element(el) = &mut widget.kind else { unreachable!("this is always a floating widget") };
                let mut layout_ctx = LayoutCtx::new(
                    self.attribute_storage,
                    self.dirty_widgets,
                    &self.viewport,
                    self.glyph_map,
                    self.force_layout,
                );

                layout_widget(el, children, values, constraints, &mut layout_ctx, true);

                // Position
                position_widget(pos, el, children, values, self.attribute_storage, true, self.viewport);

                // Paint
                self.backend
                    .paint(self.glyph_map, el, children, values, self.attribute_storage, true);
            });
        }
    }

    pub fn run(&mut self) {
        let mut filter = LayoutFilter::new(true, self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            // Layout
            let mut layout_ctx = LayoutCtx::new(
                self.attribute_storage,
                self.dirty_widgets,
                &self.viewport,
                self.glyph_map,
                self.force_layout,
            );
            layout_widget(widget, children, values, self.constraints, &mut layout_ctx, true);

            // Position
            position_widget(
                Pos::ZERO,
                widget,
                children,
                values,
                self.attribute_storage,
                true,
                self.viewport,
            );

            // Paint
            self.backend
                .paint(self.glyph_map, widget, children, values, self.attribute_storage, true);
        });

        self.floating();
    }
}
