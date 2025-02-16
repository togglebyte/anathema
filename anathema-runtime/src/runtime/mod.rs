use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{clear_all_changes, clear_all_subs, drain_changes, Changes, StateId, States};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_value_resolver::{AttributeStorage, Scope};
use anathema_widgets::components::deferred::{CommandKind, DeferredComponents};
use anathema_widgets::components::events::Event;
use anathema_widgets::components::{
    AnyComponent, AnyComponentContext, AssociatedEvents, ComponentKind, ComponentRegistry, Emitter, ViewMessage,
};
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::query::Children;
use anathema_widgets::{
    eval_blueprint, update_widget, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, WidgetId, WidgetKind,
    WidgetTree,
};
use flume::Receiver;
use notify::{INotifyWatcher, RecommendedWatcher};

use crate::builder::Builder;
pub use crate::error::Result;
use crate::events::GlobalEventHandler;
use crate::{Error, REBUILD};

mod testing;

pub struct Runtime<G> {
    pub(super) blueprint: Blueprint,
    pub(super) globals: Globals,
    pub(super) factory: Factory,
    pub(super) states: States,
    pub(super) component_registry: ComponentRegistry,
    pub(super) components: Components,
    pub(super) document: Document,
    pub(super) floating_widgets: FloatingWidgets,
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
    pub(super) global_event_handler: G,
}

impl Runtime<()> {
    pub fn builder<B: Backend>(doc: Document, backend: &B) -> Builder<()> {
        Builder::new(doc, backend.size(), ())
    }
}

impl<G: GlobalEventHandler> Runtime<G> {
    pub(crate) fn new(
        component_registry: ComponentRegistry,
        mut document: Document,
        mut err_document: Document,
        factory: Factory,
        message_receiver: Receiver<ViewMessage>,
        emitter: Emitter,
        watcher: Option<INotifyWatcher>,
        size: Size,
        fps: u32,
        global_event_handler: G,
    ) -> Result<Self> {
        let (blueprint, globals) = document.compile()?;
        let Ok((err_blueprint, err_globals)) = err_document.compile() else { panic!("the error display failed to compile") };

        let sleep_micros: u64 = ((1.0 / fps as f64) * 1000.0 * 1000.0) as u64;

        let inst = Self {
            component_registry,
            components: Components::new(),
            document,
            factory,
            states: States::new(),
            floating_widgets: FloatingWidgets::empty(),
            dirty_widgets: DirtyWidgets::empty(),
            assoc_events: AssociatedEvents::new(),
            glyph_map: GlyphMap::empty(),
            blueprint,
            globals,
            changes: Changes::empty(),
            viewport: Viewport::new(size),
            message_receiver,
            emitter,
            dt: Instant::now(),
            _watcher: watcher,
            deferred_components: DeferredComponents::new(),
            sleep_micros,
            global_event_handler,
        };
        Ok(inst)
    }

    pub fn with_frame<B: Backend, F>(&mut self, backend: &mut B, mut f: F) -> Result<()>
    where
        B: Backend,
        F: FnMut(&mut B, Frame<'_, '_, G>) -> Result<()>,
    {
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut frame = self.next_frame(&mut tree, &mut attribute_storage)?;
        frame.init_tree()?;
        f(backend, frame)
    }

    pub fn run<B: Backend>(&mut self, backend: &mut B) -> Result<()> {
        let sleep_micros = self.sleep_micros;
        self.with_frame(backend, |backend, mut frame| loop {
            frame.tick(backend)?;
            if frame.stop {
                return Err(Error::Stop);
            }

            frame.present(backend);
            frame.cleanup();
            std::thread::sleep(Duration::from_micros(sleep_micros));

            if REBUILD.swap(false, Ordering::Relaxed) {
                frame.return_state();
                break Ok(());
            }
        })?;

        Ok(())
    }

    pub fn next_frame<'frame, 'bp>(
        &'bp mut self,
        tree: &'frame mut WidgetTree<'bp>,
        attribute_storage: &'frame mut AttributeStorage<'bp>,
    ) -> Result<Frame<'frame, 'bp, G>> {
        let layout_ctx = LayoutCtx::new(
            &self.globals,
            &self.factory,
            &mut self.states,
            attribute_storage,
            &mut self.components,
            &mut self.component_registry,
            &mut self.floating_widgets,
            &mut self.glyph_map,
            &mut self.dirty_widgets,
            &mut self.viewport,
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
            needs_layout: true,
            stop: false,

            global_event_handler: &self.global_event_handler,
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

pub struct Frame<'rt, 'bp, G> {
    document: &'bp Document,
    blueprint: &'bp Blueprint,
    pub tree: &'rt mut WidgetTree<'bp>,
    deferred_components: &'rt mut DeferredComponents,
    layout_ctx: LayoutCtx<'rt, 'bp>,
    changes: &'rt mut Changes,
    assoc_events: &'rt mut AssociatedEvents,
    sleep_micros: u64,
    emitter: &'rt Emitter,
    message_receiver: &'rt flume::Receiver<ViewMessage>,
    dt: &'rt mut Instant,
    needs_layout: bool,
    stop: bool,
    global_event_handler: &'rt G,
}

impl<'bp, G: GlobalEventHandler> Frame<'_, 'bp, G> {
    pub fn handle_global_event(&mut self, event: Event) -> Option<Event> {
        self.global_event_handler.handle(event, &mut self.deferred_components)
    }

    pub fn event(&mut self, event: Event) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let Some(event) = self.handle_global_event(event) else { return };
        if let Event::Stop = event {
            self.stop = true;
            return;
        }

        // if event.is_ctrl_c() {
        //     panic!("ctrl c quit, this is a hack");
        // }

        match event {
            Event::Noop => return,
            Event::Stop => todo!(),
            Event::Blur | Event::Focus | Event::Key(_) => {
                let Some((widget_id, state_id)) = self.layout_ctx.components.current() else { return };
                self.send_event_to_component(event, widget_id, state_id);
            }
            Event::Mouse(_) | Event::Resize(_) => {
                for i in 0..self.layout_ctx.components.total_len() {
                    let Some((widget_id, state_id, _)) = self.layout_ctx.components.get(i as u32) else { continue };
                    self.send_event_to_component(event, widget_id, state_id);
                }
            }
            Event::Tick(_) => panic!("this event should never be sent to the runtime"),
        }
    }

    // Should be called only once to initialise the node tree.
    pub fn init_tree(&mut self) -> Result<()> {
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

    pub fn tick<B: Backend>(&mut self, backend: &mut B) -> Result<Duration> {
        #[cfg(feature = "profile")]
        puffin::GlobalProfiler::lock().new_frame();

        let now = Instant::now();
        self.tick_components(self.dt.elapsed());
        let elapsed = self.handle_messages(now);
        self.poll_events(elapsed, now, backend);
        self.drain_deferred_commands();
        self.drain_assoc_events();
        self.apply_changes()?;
        self.cycle(backend)?;

        *self.dt = Instant::now();
        Ok(now.elapsed())
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
            self.layout_ctx.components.try_remove(key);
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
            if fps_now.elapsed().as_micros() as u64 >= self.sleep_micros / 2 {
                break;
            }
        }

        fps_now.elapsed()
    }

    fn poll_events<B: Backend>(&mut self, remaining: Duration, fps_now: Instant, backend: &mut B) {
        while let Some(event) = backend.next_event(remaining) {
            if let Event::Resize(size) = event {
                self.layout_ctx.viewport.resize(size);
                self.needs_layout = true;
                backend.resize(size, self.layout_ctx.glyph_map);
            }

            self.event(event);

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() as u64 > self.sleep_micros {
                break;
            }
        }
    }

    fn drain_deferred_commands(&mut self) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

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
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        while let Some(event) = self.assoc_events.next() {
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

    fn cycle<B: Backend>(&mut self, backend: &mut B) -> Result<()> {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let mut cycle = WidgetCycle::new(backend, self.tree.view_mut(), self.layout_ctx.viewport.constraints());
        cycle.run(&mut self.layout_ctx, self.needs_layout)?;
        self.needs_layout = false;
        Ok(())
    }

    fn apply_changes(&mut self) -> Result<()> {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        drain_changes(self.changes);

        if self.changes.is_empty() {
            return Ok(());
        }

        self.needs_layout = true;
        let mut tree = self.tree.view_mut();
        self.changes.iter().try_for_each(|(sub, change)| {
            sub.iter().try_for_each(|value_id| {
                let widget_id = value_id.key();

                if let Some(widget) = tree.get_mut(widget_id) {
                    if let WidgetKind::Element(element) = &mut widget.kind {
                        element.invalidate_cache();
                    }
                }

                // check that the node hasn't already been removed
                if !tree.contains(widget_id) {
                    return Result::Ok(());
                }

                tree.with_value_mut(widget_id, |_path, widget, tree| {
                    update_widget(widget, value_id, change, tree, self.layout_ctx.attribute_storage)
                })?;

                Ok(())
            })?;

            Result::Ok(())
        })?;

        Ok(())
    }

    fn send_event_to_component(&mut self, event: Event, widget_id: WidgetId, state_id: StateId) {
        self.with_component(widget_id, state_id, |comp, elements, ctx| {
            comp.any_event(elements, ctx, event);
        });
    }

    fn with_component<F, U>(&mut self, widget_id: WidgetId, state_id: StateId, f: F)
    where
        F: FnOnce(&mut Box<dyn AnyComponent>, Children<'_, '_>, AnyComponentContext<'_>) -> U,
    {
        let mut tree = self.tree.view_mut();

        tree.with_value_mut(widget_id, |_path, container, children| {
            let WidgetKind::Component(component) = &mut container.kind else { return };

            let state = self.layout_ctx.states.get_mut(state_id);

            self.layout_ctx
                .attribute_storage
                .with_mut(widget_id, |attributes, storage| {
                    let elements = Children::new(children, storage, self.layout_ctx.dirty_widgets);

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
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let len = self.layout_ctx.components.total_len();
        for i in 0..len {
            let Some((widget_id, state_id, ticks)) = self.layout_ctx.components.get(i as u32) else { continue };

            if !ticks {
                continue;
            }
            let event = Event::Tick(dt);
            self.send_event_to_component(event, widget_id, state_id);
        }
    }

    // Return the state for each component back into the component registry
    fn return_state(self) {
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

    fn display_error(&mut self, backend: &mut impl Backend) {
        let tpl = "text 'you goofed up'";
        backend.render(self.layout_ctx.glyph_map);
    }
}
