use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{
    clear_all_changes, clear_all_subs, drain_changes, drain_watchers, AnyState, Changes, State, StateId, States,
    Watched, Watcher,
};
use anathema_store::stack::Stack;
use anathema_store::tree::root_node;
use anathema_strings::HStrings;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_value_resolver::{AttributeStorage, Scope};
use anathema_widgets::components::deferred::{CommandKind, DeferredComponents};
use anathema_widgets::components::events::{Event, KeyEvent};
use anathema_widgets::components::{
    AnyComponent, AnyComponentContext, AnyEventCtx, AssociatedEvents, ComponentContext, ComponentKind,
    ComponentRegistry, Emitter, UntypedContext, ViewMessage,
};
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::query::{Children, Elements};
use anathema_widgets::{
    eval_blueprint, update_widget, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, WidgetId,
    WidgetKind, WidgetTree, WidgetTreeView,
};
use notify::RecommendedWatcher;

use crate::builder::Builder;
pub use crate::error::Result;
use crate::REBUILD;

mod testing;

pub struct Runtime {
    pub(super) blueprint: Blueprint,
    pub(super) globals: Globals,
    pub(super) factory: Factory,
    pub(super) states: States,
    pub(super) component_registry: ComponentRegistry,
    pub(super) components: Components,
    pub(super) document: Document,
    pub(super) floating_widgets: FloatingWidgets,
    pub(super) changelist: ChangeList,
    pub(super) dirty_widgets: DirtyWidgets,
    pub(super) assoc_events: AssociatedEvents,
    pub(super) glyph_map: GlyphMap,
    pub(super) changes: Changes,
    pub(super) viewport: Viewport,
    pub(super) emitter: Emitter,
    pub(super) sleep_micros: u64,
    pub(super) message_receiver: flume::Receiver<ViewMessage>,
    pub(super) dt: Instant,
    pub(super) _watcher: Option<RecommendedWatcher>,
    pub(super) deferred_components: DeferredComponents,
}

impl Runtime {
    pub fn builder(doc: Document) -> Builder {
        Builder::new(doc)
    }

    pub fn run<B: Backend>(&mut self, backend: &mut B) -> Result<()> {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let sleep_micros = self.sleep_micros;
        let mut frame = self.next_frame(&mut tree, &mut attribute_storage)?;
        frame.init();
        frame.tick(backend, true);

        loop {
            frame.tick(backend, false);
            frame.present(backend);
            frame.cleanup();
            std::thread::sleep(Duration::from_micros(sleep_micros));

            if REBUILD.swap(false, Ordering::Relaxed) {
                frame.return_state();
                break;
            }
        }

        Ok(())
    }

    pub fn next_frame<'frame, 'bp>(
        &'bp mut self,
        tree: &'frame mut WidgetTree<'bp>,
        attribute_storage: &'frame mut AttributeStorage<'bp>,
    ) -> Result<Frame<'frame, 'bp>> {
        #[cfg(feature = "profile")]
        puffin::GlobalProfiler::lock().new_frame();

        let layout_ctx = LayoutCtx::new(
            &self.globals,
            &self.factory,
            &mut self.states,
            attribute_storage,
            &mut self.components,
            &mut self.component_registry,
            &mut self.floating_widgets,
            &mut self.changelist,
            &mut self.glyph_map,
            &mut self.dirty_widgets,
            &mut self.viewport,
            false,
        );

        let inst = Frame {
            document: &self.document,
            blueprint: &self.blueprint,
            tree,
            layout_ctx,
            changes: &mut self.changes,
            sleep_micros: self.sleep_micros,

            assoc_events: &mut self.assoc_events,
            deferred_components: &mut self.deferred_components,

            emitter: &self.emitter,
            message_receiver: &self.message_receiver,

            dt: &mut self.dt,
        };

        Ok(inst)
    }

    pub(super) fn reload(&mut self) -> Result<()> {
        clear_all_changes();
        clear_all_subs();

        self.components = Components::new();
        self.floating_widgets = FloatingWidgets::empty();

        // Reload templates
        self.document.reload_templates()?;

        let (blueprint, globals) = self.document.compile()?;
        self.blueprint = blueprint;
        self.globals = globals;

        Ok(())
    }
}

pub struct Frame<'rt, 'bp> {
    document: &'bp Document,
    blueprint: &'bp Blueprint,
    tree: &'rt mut WidgetTree<'bp>,
    deferred_components: &'rt mut DeferredComponents,
    layout_ctx: LayoutCtx<'rt, 'bp>,
    changes: &'rt mut Changes,
    assoc_events: &'rt mut AssociatedEvents,
    sleep_micros: u64,
    emitter: &'rt Emitter,
    message_receiver: &'rt flume::Receiver<ViewMessage>,
    dt: &'rt mut Instant,
}

impl<'bp> Frame<'_, 'bp> {
    pub fn event(&mut self, event: Event) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        match event {
            Event::Noop => return,
            Event::Stop => todo!(),
            Event::Blur | Event::Focus | Event::Key(_) => {
                let Some((widget_id, state_id)) = self.layout_ctx.components.current() else { return };
                self.send_event_to_component(event, widget_id, state_id);
            }
            Event::Mouse(_) | Event::Resize(_) => {
                for i in 0..self.layout_ctx.components.len() {
                    let Some((widget_id, state_id)) = self.layout_ctx.components.get(i as u32) else { continue };
                    self.send_event_to_component(event, widget_id, state_id);
                }
            }
            Event::Tick(_) => panic!("this event should never be sent to the runtime"),
        }
    }

    // Should be called only once to initialise the node tree.
    pub fn init(&mut self) -> Result<()> {
        let mut ctx = self.layout_ctx.eval_ctx(None);
        eval_blueprint(
            self.blueprint,
            &mut ctx,
            &Scope::root(),
            root_node(),
            &mut self.tree.view_mut(),
        )?;
        Ok(())
    }

    pub fn tick<B: Backend>(&mut self, backend: &mut B, force_layout: bool) -> Duration {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        self.layout_ctx.force_layout = force_layout;

        let now = Instant::now();
        self.tick_components(self.dt.elapsed());
        let elapsed = self.handle_messages(now);
        self.poll_events(elapsed, now, backend);
        self.drain_deferred_commands();
        self.drain_assoc_events();
        self.apply_changes();
        self.cycle(backend);

        // reset force_layout
        self.layout_ctx.force_layout = false;

        *self.dt = Instant::now();
        now.elapsed()
    }

    pub fn present<B: Backend>(&mut self, backend: &mut B) -> Duration {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let now = Instant::now();
        backend.render(self.layout_ctx.glyph_map);
        backend.clear();
        now.elapsed()
    }

    pub fn cleanup(&mut self) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        self.changes.clear();
        self.layout_ctx.dirty_widgets.clear();

        for key in self.tree.drain_removed() {
            self.layout_ctx.attribute_storage.try_remove(key);
            self.layout_ctx.floating_widgets.try_remove(key);
            self.layout_ctx.components.remove(key);
        }
    }

    fn handle_messages(&mut self, fps_now: Instant) -> Duration {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        while let Ok(msg) = self.message_receiver.try_recv() {
            if let Some((widget_id, state_id)) = self
                .layout_ctx
                .components
                .get_by_component_id(msg.recipient())
                .map(|e| (e.widget_id, e.state_id))
            {
                self.with_component(widget_id, state_id, |comp, elements, ctx| {
                    comp.any_message(elements, ctx, msg.payload())
                });
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() as u64 > self.sleep_micros / 2 {
                break;
            }
        }

        fps_now.elapsed()
    }

    fn poll_events<B: Backend>(&mut self, remaining: Duration, fps_now: Instant, backend: &mut B) {
        while let Some(event) = backend.next_event(remaining) {
            if let Event::Resize(size) = event {
                self.layout_ctx.viewport.resize(size);
                self.layout_ctx.force_layout = true;
                backend.resize(size, self.layout_ctx.glyph_map);
            }

            if event.is_ctrl_c() {
                panic!("ctrl c quit, this is a hack");
            }

            self.event(event);

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() as u64 > self.sleep_micros {
                break;
            }
        }
    }

    fn drain_deferred_commands(&mut self) {
        // TODO don't cause a bunch of allocations here.
        // Possibly drain into a new list
        let commands = self.deferred_components.drain().collect::<Vec<_>>();
        for mut cmd in commands {
            let Some((widget_id, state_id)) =
                cmd.filter_components(self.tree.view_mut(), self.layout_ctx.attribute_storage)
            else {
                continue;
            };

            match cmd.kind {
                CommandKind::SendMessage(msg) => {
                    self.with_component(widget_id, state_id, |comp, elements, ctx| {
                        comp.any_message(elements, ctx, msg)
                    });
                }
                CommandKind::Focus => {
                    // Blur old focus
                    if let Some((widget_id, state_id)) = self.layout_ctx.components.current() {
                        self.with_component(widget_id, state_id, |comp, children, ctx| comp.any_blur(children, ctx));
                    }

                    // Set new focus
                    self.layout_ctx.components.set(widget_id);
                    self.with_component(widget_id, state_id, |comp, children, ctx| comp.any_focus(children, ctx));
                }
            }
        }
    }

    fn drain_assoc_events(&mut self) {
        while let Some(mut event) = self.assoc_events.next() {
            let Some((widget_id, state_id)) = self.layout_ctx.components.get_by_widget_id(event.parent.into()) else {
                return;
            };

            let Some(remote_state) = self.layout_ctx.states.get(event.state) else { return };
            let Some(remote_state) = remote_state.shared_state() else { return };
            self.with_component(widget_id, state_id, |comp, children, ctx| {
                let event_ident = self.document.strings.get_ref_unchecked(event.external);
                comp.any_receive(children, ctx, event_ident, &*remote_state);
            });
        }
    }

    fn cycle<B: Backend>(&mut self, backend: &mut B) {
        let mut cycle = WidgetCycle::new(backend, self.tree, self.layout_ctx.viewport.constraints());
        cycle.run(&mut self.layout_ctx);
    }

    fn apply_changes(&mut self) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        drain_changes(self.changes);

        if self.changes.is_empty() {
            return;
        }

        let mut tree = self.tree.view_mut();
        self.changes.iter().for_each(|(sub, change)| {
            sub.iter().for_each(|value_id| {
                let widget_id = value_id.key();
                self.layout_ctx.dirty_widgets.push(widget_id);

                // check that the node hasn't already been removed
                if !tree.contains(widget_id) {
                    return;
                }

                tree.with_value_mut(widget_id, |path, widget, tree| {
                    update_widget(widget, value_id, change, path, tree, self.layout_ctx.attribute_storage);
                });
            });
        });
    }

    fn send_event_to_component(&mut self, event: Event, widget_id: WidgetId, state_id: StateId) {
        self.with_component(widget_id, state_id, |comp, elements, ctx| {
            if !comp.any_ticks() && matches!(event, Event::Tick(_)) {
                return;
            }

            comp.any_event(elements, ctx, event);
        });
    }

    fn with_component<F, U>(&mut self, widget_id: WidgetId, state_id: StateId, mut f: F)
    where
        F: FnOnce(&mut Box<dyn AnyComponent>, Children<'_, '_>, AnyComponentContext<'_>) -> U,
    {
        let mut tree = self.tree.view_mut();

        tree.with_value_mut(widget_id, |path, container, children| {
            let WidgetKind::Component(component) = &mut container.kind else { return };

            let state = self.layout_ctx.states.get_mut(state_id);

            self.layout_ctx
                .attribute_storage
                .with_mut(widget_id, |attributes, storage| {
                    let mut elements = Children::new(children, storage, self.layout_ctx.dirty_widgets);

                    let Some(state) = state else { return };

                    let ctx = AnyComponentContext::new(
                        component.parent.map(Into::into),
                        state_id,
                        component.assoc_functions,
                        self.assoc_events,
                        self.deferred_components,
                        attributes,
                        Some(state),
                        self.emitter,
                        self.layout_ctx.viewport,
                        &self.document.strings,
                    );

                    f(&mut component.dyn_component, elements, ctx);
                });
        });
    }

    fn tick_components(&mut self, dt: Duration) {
        for i in 0..self.layout_ctx.components.len() {
            let Some((widget_id, state_id)) = self.layout_ctx.components.get(i as u32) else { continue };
            let event = Event::Tick(dt);
            self.send_event_to_component(event, widget_id, state_id);
        }
    }

    // Return the state for each component back into the component registry
    fn return_state(mut self) {
        // Return all states
        let mut tree = WidgetTree::empty();
        std::mem::swap(&mut tree, self.tree);

        for (_, widget) in tree.values().into_iter() {
            let WidgetKind::Component(comp) = widget.kind else { continue };
            let ComponentKind::Instance = comp.kind else { continue };
            let state = self.layout_ctx.states.remove(comp.state_id).take();
            self.layout_ctx
                .component_registry
                .return_component(comp.component_id, comp.dyn_component, state);
        }
    }
}
