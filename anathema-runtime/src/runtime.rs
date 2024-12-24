// -----------------------------------------------------------------------------
//   - Notes on viewstructs -
//   Well, it's not really about viewstructs but here goes:
//   * Split the runtime into different sections with different borrowed values
// -----------------------------------------------------------------------------

use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{drain_changes, drain_futures, AnyState, Changes, FutureValues, State, StateId, States};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::events::Event;
use anathema_widgets::components::ComponentRegistry;
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::{
    eval_blueprint, update_widget, AttributeStorage, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets,
    GlyphMap, Scope, WidgetTree, WidgetTreeView,
};

pub use crate::error::Result;

pub struct Runtime<'bp> {
    pub(super) blueprint: &'bp Blueprint,
    pub(super) globals: &'bp Globals,
    pub(super) factory: &'bp Factory,
    pub(super) tree: WidgetTree<'bp>,
    pub(super) states: States,
    pub(super) attribute_storage: AttributeStorage<'bp>,
    pub(super) component_registry: &'bp mut ComponentRegistry,
    pub(super) components: Components,
    pub(super) document: &'bp mut Document,
    pub(super) floating_widgets: FloatingWidgets,
    pub(super) changelist: ChangeList,
    pub(super) dirty_widgets: DirtyWidgets,
    pub(super) future_values: FutureValues,
    pub(super) glyph_map: GlyphMap,
    pub(super) changes: Changes,
    pub(super) viewport: Viewport,
}

impl<'bp> Runtime<'bp> {
    pub fn next_frame(&mut self) -> Result<Frame<'_, 'bp>> {
        let layout_ctx = LayoutCtx::new(
            self.globals,
            &self.factory,
            &mut self.states,
            &mut self.attribute_storage,
            &mut self.components,
            &mut self.component_registry,
            &mut self.floating_widgets,
            &mut self.changelist,
            &mut self.glyph_map,
            &mut self.dirty_widgets,
            self.viewport,
            true,
        );

        let inst = Frame {
            document: self.document,
            tree: &mut self.tree,
            layout_ctx,
            changes: &mut self.changes,
            future_values: &mut self.future_values,
        };

        Ok(inst)
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        let blueprint = self.blueprint;
        let mut first_frame = self.next_frame()?;
        first_frame.init(blueprint);
        Ok(())
    }

    pub fn select_component(&mut self) {
        // self.components
    }

    pub fn state_id(&mut self, component_id: usize) -> Option<StateId> {
        self.components.get(component_id).map(|(_, id)| id)
    }

    pub fn get_state(&mut self, state_id: StateId) -> Option<&dyn AnyState> {
        self.states.get(state_id)
    }
}

pub struct Frame<'rt, 'bp> {
    // backend: &'rt mut B,
    document: &'rt mut Document,
    tree: &'rt mut WidgetTree<'bp>,
    layout_ctx: LayoutCtx<'rt, 'bp>,
    changes: &'rt mut Changes,
    future_values: &'rt mut FutureValues,
}

impl<'bp> Frame<'_, 'bp> {
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Noop => return,
            Event::Stop => todo!(),
            Event::Blur => todo!(),
            Event::Focus => todo!(),
            Event::Key(key_event) => {}
            Event::Mouse(mouse_event) => {
                // for i in 0..self.eval_ctx.components.len() {
                //     let (widget_id, state_id) = self
                //         .eval_ctx
                //         .components
                //         .get(i)
                //         .expect("components can not change during this call");

                //     // tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_event(ctx, event));
                // }
            }
            Event::Resize(size) => todo!(),
        }
    }

    // Should be called only once to initialise the node tree.
    fn init(&mut self, blueprint: &'bp Blueprint) -> Result<()> {
        let mut ctx = self.layout_ctx.eval_ctx();
        eval_blueprint(blueprint, &mut ctx, root_node(), &mut self.tree.view_mut())?;
        Ok(())
    }

    pub fn tick<B: Backend>(&mut self, backend: &mut B) -> Duration {
        let now = Instant::now();
        self.apply_changes();
        self.resolve_future_values();
        let mut cycle = WidgetCycle::new(backend, self.tree, self.layout_ctx.viewport.constraints());
        cycle.run(&mut self.layout_ctx);
        now.elapsed()
    }

    pub fn present<B: Backend>(&mut self, backend: &mut B) -> Duration {
        let now = Instant::now();
        backend.render(self.layout_ctx.glyph_map);
        backend.clear();
        now.elapsed()
    }

    pub fn cleanup(&mut self) {
        self.changes.clear();
        self.layout_ctx.dirty_widgets.clear();

        for key in self.tree.drain_removed() {
            self.layout_ctx.attribute_storage.try_remove(key);
            self.layout_ctx.floating_widgets.try_remove(key);
            self.layout_ctx.components.remove(key);
        }
    }

    fn apply_changes(&mut self) {
        drain_changes(self.changes);

        if self.changes.is_empty() {
            return;
        }

        self.changes.iter().for_each(|(sub, change)| {
            sub.iter().for_each(|sub| {
                self.layout_ctx.dirty_widgets.push(sub.key());
                self.layout_ctx.changelist.insert(sub.key(), sub);

                let mut tree = self.tree.view_mut();
                tree.with_value_mut(sub.key(), |path, widget, tree| {
                    update_widget(widget, sub, change, path, tree);
                });
            });
        });
    }

    fn resolve_future_values(&mut self) {
        drain_futures(&mut self.future_values);

        if self.future_values.is_empty() {
            return;
        }

        for sub in self.future_values.drain().rev() {
            self.layout_ctx.changelist.insert(sub.key(), sub);
            self.layout_ctx.dirty_widgets.push(sub.key());
        }
    }

    fn poll_event<B: Backend>(&mut self, poll_timeout: Duration, backend: &mut B) {
        let Some(event) = backend.next_event(poll_timeout) else { return };
        self.event(event);
    }
}
