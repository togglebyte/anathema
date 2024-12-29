use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{drain_changes, drain_futures, AnyState, Changes, FutureValues, State, StateId, States};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::events::Event;
use anathema_widgets::components::{
    AnyEventCtx, AssociatedEvents, ComponentContext, ComponentRegistry, Emitter, FocusQueue, UntypedContext,
    ViewMessage,
};
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::{
    eval_blueprint, update_widget, AttributeStorage, ChangeList, Components, DirtyWidgets, Elements, Factory,
    FloatingWidgets, GlyphMap, Scope, WidgetId, WidgetKind, WidgetTree, WidgetTreeView,
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
    pub(super) assoc_events: AssociatedEvents,
    pub(super) focus_queue: FocusQueue<'static>,
    pub(super) glyph_map: GlyphMap,
    pub(super) changes: Changes,
    pub(super) viewport: Viewport,
    pub(super) emitter: Emitter,
    pub(super) fps: usize,
    pub(super) sleep_micros: u128,
    pub(super) message_receiver: flume::Receiver<ViewMessage>,
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
            sleep_micros: self.sleep_micros,

            focus_queue: &mut self.focus_queue,
            assoc_events: &mut self.assoc_events,

            emitter: &self.emitter,
            message_receiver: &self.message_receiver,
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
    document: &'rt mut Document,
    tree: &'rt mut WidgetTree<'bp>,
    layout_ctx: LayoutCtx<'rt, 'bp>,
    changes: &'rt mut Changes,
    future_values: &'rt mut FutureValues,
    assoc_events: &'rt mut AssociatedEvents,
    focus_queue: &'rt mut FocusQueue<'static>,
    sleep_micros: u128,
    emitter: &'rt Emitter,
    message_receiver: &'rt flume::Receiver<ViewMessage>,
}

impl<'bp> Frame<'_, 'bp> {
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Noop => return,
            Event::Stop => todo!(),
            Event::Blur | Event::Focus | Event::Key(_) => {
                let Some((widget_id, state_id)) = self.layout_ctx.components.current() else { return };
                self.send_event_to_component(event, widget_id, state_id);
            }
            Event::Mouse(mouse_event) => {
                for i in 0..self.layout_ctx.components.len() {
                    let (widget_id, state_id) = self
                        .layout_ctx
                        .components
                        .get(i)
                        .expect("components can not change during this call");

                    self.send_event_to_component(event, widget_id, state_id);
                }
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
        let elapsed = self.handle_messages(now);
        self.pull_events(elapsed, now, backend);
        self.apply_changes();
        self.resolve_future_values();
        self.cycle(backend);
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

    fn handle_messages(&mut self, fps_now: Instant) -> Duration {
        while let Ok(msg) = self.message_receiver.try_recv() {
            if let Some((widget_id, state_id)) = self
                .layout_ctx
                .components
                .get_by_component_id(msg.recipient())
                .map(|e| (e.widget_id, e.state_id))
            {
                // tree.with_component(widget_id, state_id, &mut event_ctx, |a, b| {
                //     a.any_message(msg.payload(), b)
                // });
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > self.sleep_micros / 2 {
                break;
            }
        }

        fps_now.elapsed()
    }

    fn pull_events<B: Backend>(&mut self, remaining: Duration, fps_now: Instant, backend: &mut B) {
        while let Some(event) = backend.next_event(remaining) {
            self.event(event);

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > self.sleep_micros {
                break;
            }
        }
    }

    fn cycle<B: Backend>(&mut self, backend: &mut B) {
        let mut cycle = WidgetCycle::new(backend, self.tree, self.layout_ctx.viewport.constraints());
        cycle.run(&mut self.layout_ctx);
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

    fn send_event_to_component(&mut self, event: Event, widget_id: WidgetId, state_id: StateId) {
        let mut tree = self.tree.view_mut();

        tree.with_value_mut(widget_id, |path, container, children| {
            let WidgetKind::Component(component) = &mut container.kind else { return };
            let state = self.layout_ctx.states.get_mut(state_id);

            self.layout_ctx
                .attribute_storage
                .with_mut(widget_id, |attributes, storage| {
                    let component_ctx = ComponentContext::new(
                        state_id,
                        component.parent,
                        component.assoc_functions,
                        self.assoc_events,
                        self.focus_queue,
                        attributes,
                    );

                    let mut elements = Elements::new(children, storage, self.layout_ctx.dirty_widgets);

                    let event_ctx = AnyEventCtx {
                        state,
                        elements,
                        context: UntypedContext {
                            emitter: self.emitter,
                            viewport: self.layout_ctx.viewport,
                            strings: &self.document.strings,
                        },
                        component_ctx,
                    };

                    component.dyn_component.any_event(event_ctx, event);
                });
        });
    }
}
