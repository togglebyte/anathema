use std::time::Duration;

use anathema_geometry::{Pos, Size};
use anathema_store::tree::{AsNodePath, Node, TreeValues};
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{AttributeStorage, Element, FloatingWidgets, WidgetKind, WidgetTree};

pub mod test;
pub mod tui;

pub trait Backend {
    fn size(&self) -> Size;

    fn next_event(&mut self, timeout: Duration) -> Option<Event>;

    fn resize(&mut self, new_size: Size);

    /// Paint the widgets
    fn paint<'bp>(
        &mut self,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetKind<'bp>>,
        attribute_storage: &AttributeStorage<'bp>,
        ignore_floats: bool,
    );

    /// Called by the runtime at the end of the frame.
    fn render(&mut self);

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
    attribute_storage: &'rt AttributeStorage<'bp>,
    floating_widgets: &'rt FloatingWidgets,
    viewport: Viewport,
}

impl<'rt, 'bp, T: Backend> WidgetCycle<'rt, 'bp, T> {
    pub fn new(
        backend: &'rt mut T,
        tree: &'rt mut WidgetTree<'bp>,
        constraints: Constraints,
        attribute_storage: &'rt AttributeStorage<'bp>,
        floating_widgets: &'rt FloatingWidgets,
        viewport: Viewport,
    ) -> Self {
        Self {
            backend,
            tree,
            constraints,
            attribute_storage,
            floating_widgets,
            viewport,
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
                        Some(WidgetKind::Element(el)) => {
                            let bounds = el.inner_bounds();
                            break (bounds.start, Constraints::from(bounds));
                        }
                        _ => parent = p.parent(),
                    },
                }
            };

            self.tree.with_nodes_and_values(*widget_id, |widget, children, values| {
                let WidgetKind::Element(el) = widget else { unreachable!("this is always a floating widget") };
                let mut layout_ctx = LayoutCtx::new(self.attribute_storage, &self.viewport);

                layout_widget(el, children, values, constraints, &mut layout_ctx, true);

                // Position
                position_widget(pos, el, children, values, self.attribute_storage, true, self.viewport);

                // Paint
                self.backend.paint(el, children, values, self.attribute_storage, true);
            });
        }
    }

    pub fn run(&mut self) {
        let mut filter = LayoutFilter::new(true, self.attribute_storage);
        self.tree.for_each(&mut filter).first(&mut |widget, children, values| {
            // Layout
            // TODO: once the text buffer can be read-only for the paint
            //       the context can be made outside of this closure.
            //
            //       That doesn't have as much of an impact here
            //       as it will do when dealing with the floating widgets
            let mut layout_ctx = LayoutCtx::new(self.attribute_storage, &self.viewport);
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
                .paint(widget, children, values, self.attribute_storage, true);
        });

        self.floating();
    }
}
