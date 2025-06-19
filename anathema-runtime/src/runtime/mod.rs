use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_geometry::Size;
use anathema_state::{Changes, StateId, States, clear_all_changes, clear_all_subs, drain_changes};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_value_resolver::{AttributeStorage, FunctionTable, Scope};
use anathema_widgets::components::deferred::{CommandKind, DeferredComponents};
use anathema_widgets::components::events::Event;
use anathema_widgets::components::{
    AnyComponentContext, AssociatedEvents, ComponentKind, ComponentRegistry, Emitter, ViewMessage,
};
use anathema_widgets::layout::{LayoutCtx, Viewport};
use anathema_widgets::query::Children;
use anathema_widgets::tabindex::{Index, TabIndex};
use anathema_widgets::{
    Component, Components, Factory, FloatingWidgets, GlyphMap, WidgetContainer, WidgetId, WidgetKind, WidgetTree,
    eval_blueprint, update_widget,
};
use flume::Receiver;
use notify::RecommendedWatcher;

pub(crate) use self::error::show_error;
use crate::builder::Builder;
pub use crate::error::Result;
use crate::events::GlobalEventHandler;
use crate::{Error, REBUILD};

mod error;
mod testing;

/// Anathema runtime
pub struct Runtime<G> {
    pub(super) blueprint: Blueprint,
    pub(super) globals: Globals,
    pub(super) factory: Factory,
    pub(super) states: States,
    pub(super) component_registry: ComponentRegistry,
    pub(super) components: Components,
    pub(super) document: Document,
    pub(super) floating_widgets: FloatingWidgets,
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
    pub(super) function_table: FunctionTable,
}

impl Runtime<()> {
    /// Create a runtime builder
    pub fn builder<B: Backend>(doc: Document, backend: &B) -> Builder<()> {
        Builder::new(doc, backend.size(), ())
    }
}

impl<G: GlobalEventHandler> Runtime<G> {
    pub(crate) fn new(
        blueprint: Blueprint,
        globals: Globals,
        component_registry: ComponentRegistry,
        document: Document,
        factory: Factory,
        message_receiver: Receiver<ViewMessage>,
        emitter: Emitter,
        watcher: Option<RecommendedWatcher>,
        size: Size,
        fps: u32,
        global_event_handler: G,
        function_table: FunctionTable,
    ) -> Self {
        let sleep_micros: u64 = ((1.0 / fps as f64) * 1000.0 * 1000.0) as u64;

        Self {
            component_registry,
            components: Components::new(),
            document,
            factory,
            states: States::new(),
            floating_widgets: FloatingWidgets::empty(),
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
            function_table,
        }
    }

    // TODO
    // Rename Frame as it does not represent an individual frame
    // but rather something that can continuously draw.
    pub fn with_frame<B, F>(&mut self, backend: &mut B, mut f: F) -> Result<()>
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
        self.with_frame(backend, |backend, mut frame| {
            // Perform the initial tick so tab index has a tree to work with.
            // This means we can not react to any events in this tick as the tree does not
            // yet have any widgets or components.
            frame.tick(backend)?;

            let mut tabindex = TabIndex::new(&mut frame.tabindex, frame.tree.view_mut());
            tabindex.next();

            if let Some(current) = frame.tabindex.as_ref() {
                frame.with_component(current.widget_id, current.state_id, |comp, children, ctx| {
                    comp.dyn_component.any_focus(children, ctx)
                });
            }

            loop {
                frame.tick(backend)?;
                if frame.stop {
                    return Err(Error::Stop);
                }

                frame.present(backend);
                frame.cleanup();
                std::thread::sleep(Duration::from_micros(sleep_micros));

                if REBUILD.swap(false, Ordering::Relaxed) {
                    frame.force_rebuild()?;
                    backend.clear();
                    break Ok(());
                }
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
            &mut self.viewport,
            &self.function_table,
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
            tabindex: None,
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
    pub tabindex: Option<Index>,
}

impl<'rt, 'bp, G: GlobalEventHandler> Frame<'rt, 'bp, G> {
    pub fn force_rebuild(mut self) -> Result<()> {
        // call unmount on all components
        for i in 0..self.layout_ctx.components.len() {
            let Some((widget_id, state_id)) = self.layout_ctx.components.get_ticking(i) else { continue };
            let event = Event::Unmount;
            self.send_event_to_component(event, widget_id, state_id);
        }

        self.return_state_and_component();
        Ok(())
    }

    pub fn handle_global_event(&mut self, event: Event) -> Option<Event> {
        let mut tabindex = TabIndex::new(&mut self.tabindex, self.tree.view_mut());

        let event = self
            .global_event_handler
            .handle(event, &mut tabindex, self.deferred_components);

        if tabindex.changed {
            let prev = tabindex.consume();
            if let Some(prev) = prev {
                self.with_component(prev.widget_id, prev.state_id, |comp, children, ctx| {
                    comp.dyn_component.any_blur(children, ctx)
                });
            }

            if let Some(current) = self.tabindex.as_ref() {
                self.with_component(current.widget_id, current.state_id, |comp, children, ctx| {
                    comp.dyn_component.any_focus(children, ctx)
                });
            }
        }

        event
    }

    pub fn event(&mut self, event: Event) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        let Some(event) = self.handle_global_event(event) else { return };
        if let Event::Stop = event {
            self.stop = true;
            return;
        }

        match event {
            Event::Noop => (),
            Event::Stop => todo!(),

            // Component specific event
            Event::Blur | Event::Focus | Event::Key(_) => {
                let Some(Index {
                    widget_id, state_id, ..
                }) = self.tabindex
                else {
                    return;
                };
                self.send_event_to_component(event, widget_id, state_id);
            }
            Event::Mouse(_) | Event::Resize(_) => {
                for i in 0..self.layout_ctx.components.len() {
                    let Some((widget_id, state_id)) = self.layout_ctx.components.get(i) else { continue };
                    self.send_event_to_component(event, widget_id, state_id);
                }
            }
            Event::Tick(_) | Event::Mount | Event::Unmount => panic!("this event should never be sent to the runtime"),
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
        self.cycle(backend)?;
        self.init_new_components();
        self.tick_components(self.dt.elapsed());
        let elapsed = self.handle_messages(now);
        self.poll_events(elapsed, now, backend);
        self.drain_deferred_commands();
        self.drain_assoc_events();
        self.apply_changes()?;
        // TODO: this secondary call is here to deal with changes causing changes
        //       which happens when values are removed or inserted and indices needs updating
        self.apply_changes()?;

        *self.dt = Instant::now();

        match self.layout_ctx.stop_runtime {
            false => Ok(now.elapsed()),
            true => Err(Error::Stop),
        }
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
        for key in self.tree.drain_removed() {
            self.layout_ctx.attribute_storage.try_remove(key);
            self.layout_ctx.floating_widgets.try_remove(key);
            self.layout_ctx.components.try_remove(key);
            if let Some(Index { widget_id, .. }) = self.tabindex {
                if widget_id == key {
                    self.tabindex.take();
                }
            }
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
                    comp.dyn_component.any_message(elements, ctx, msg.payload())
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

        // TODO: let's keep some memory around to drain this into instead of allocating
        // a new vector every time.
        // E.g `self.deferred_components.drain_into(&mut self.deferred_buffer)`
        // Nb: Add drain_into to DeferredComponents
        let commands = self.deferred_components.drain().collect::<Vec<_>>();
        for mut cmd in commands {
            for index in 0..self.layout_ctx.components.len() {
                let Some((widget_id, state_id)) = self.layout_ctx.components.get(index) else { continue };
                let Some(comp) = self.tree.get_ref(widget_id) else { continue };
                let WidgetContainer {
                    kind: WidgetKind::Component(comp),
                    ..
                } = comp
                else {
                    continue;
                };
                let attributes = self.layout_ctx.attribute_storage.get(widget_id);
                if !cmd.filter_component(comp, attributes) {
                    continue;
                }

                // -----------------------------------------------------------------------------
                //   - Set focus -
                //   TODO: here is another candidate for refactoring to make it
                //   less cludgy and verbose.
                // -----------------------------------------------------------------------------
                // Blur the current component if the message is a `Focus` message
                if let CommandKind::Focus = cmd.kind {
                    // If this component current has focus ignore this command
                    if let Some(index) = self.tabindex.as_ref() {
                        if index.widget_id == widget_id {
                            continue;
                        }
                    }

                    // here we can find the component that should receive focus
                    let new_index = self
                        .with_component(widget_id, state_id, |comp, children, ctx| {
                            if comp.dyn_component.any_accept_focus() {
                                let index = Index {
                                    path: children.parent_path().into(),
                                    index: comp.tabindex,
                                    widget_id,
                                    state_id,
                                };

                                comp.dyn_component.any_focus(children, ctx);

                                Some(index)
                            } else {
                                None
                            }
                        })
                        .flatten();

                    if let Some(index) = new_index {
                        // If there is currently a component with focus that component
                        // should only lose focus if the selected component accepts focus.
                        if let Some(old) = self.tabindex.replace(index) {
                            self.with_component(old.widget_id, old.state_id, |comp, children, ctx| {
                                comp.dyn_component.any_blur(children, ctx)
                            });
                        }
                    }
                }

                // -----------------------------------------------------------------------------
                //   - Send message -
                // -----------------------------------------------------------------------------
                if let CommandKind::SendMessage(msg) = cmd.kind {
                    self.with_component(widget_id, state_id, |comp, children, ctx| {
                        comp.dyn_component.any_message(children, ctx, msg);
                    });
                }
                break;
            }
        }
    }

    fn drain_assoc_events(&mut self) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        while let Some(assoc_event) = self.assoc_events.next() {
            let mut parent = assoc_event.parent;
            let external_ident = self.document.strings.get_ref_unchecked(assoc_event.external());
            let internal_ident = self.document.strings.get_ref_unchecked(assoc_event.internal());
            let sender = self.document.strings.get_ref_unchecked(assoc_event.sender);
            let sender_id = assoc_event.sender_id;
            let mut event = assoc_event.to_event(internal_ident, external_ident, sender, sender_id);

            loop {
                let Some((widget_id, state_id)) = self.layout_ctx.components.get_by_widget_id(parent.into()) else {
                    return;
                };

                let stop_propagation = self
                    .with_component(widget_id, state_id, |comp, children, ctx| {
                        let next_parent = ctx.parent();
                        comp.dyn_component.any_component_event(children, ctx, &mut event);

                        parent = match next_parent {
                            Some(p) => p,
                            None => return true,
                        };

                        event.should_stop_propagation()
                    })
                    .unwrap_or(true);

                if stop_propagation {
                    break;
                }
            }
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
                    let kind = &widget.kind;
                    match kind {
                        WidgetKind::Element(_element) => {}
                        WidgetKind::For(_forloop) => {}
                        WidgetKind::Iteration(_) => {}
                        _ => (), // WidgetKind::ControlFlow(control_flow) => todo!(),
                                 // WidgetKind::ControlFlowContainer(_) => todo!(),
                                 // WidgetKind::Component(component) => todo!(),
                                 // WidgetKind::Slot => todo!(),
                    }
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
                })
                .unwrap_or(Ok(()))?;

                Ok(())
            })?;

            Result::Ok(())
        })?;

        self.changes.clear();

        Ok(())
    }

    fn send_event_to_component(&mut self, event: Event, widget_id: WidgetId, state_id: StateId) {
        self.with_component(widget_id, state_id, |comp, elements, ctx| {
            comp.dyn_component.any_event(elements, ctx, event);
        });
    }

    fn with_component<F, U>(&mut self, widget_id: WidgetId, state_id: StateId, f: F) -> Option<U>
    where
        F: FnOnce(&mut Component<'_>, Children<'_, '_>, AnyComponentContext<'_, '_>) -> U,
    {
        let mut tree = self.tree.view_mut();

        tree.with_value_mut(widget_id, |_path, container, children| {
            let WidgetKind::Component(component) = &mut container.kind else {
                panic!("this is always a component")
            };

            let Some(state) = self.layout_ctx.states.get_mut(state_id) else {
                panic!("a component always has a state")
            };

            self.layout_ctx
                .attribute_storage
                .with_mut(widget_id, |attributes, storage| {
                    let elements = Children::new(children, storage, &mut self.needs_layout);

                    let ctx = AnyComponentContext::new(
                        component.parent.map(Into::into),
                        component.name_id,
                        widget_id,
                        state_id,
                        component.assoc_functions,
                        self.assoc_events,
                        self.deferred_components,
                        attributes,
                        Some(state),
                        self.emitter,
                        self.layout_ctx.viewport,
                        &mut self.layout_ctx.stop_runtime,
                        &self.document.strings,
                    );

                    f(component, elements, ctx)
                })
        })?
    }

    fn tick_components(&mut self, dt: Duration) {
        #[cfg(feature = "profile")]
        puffin::profile_function!();

        for i in 0..self.layout_ctx.components.len() {
            let Some((widget_id, state_id)) = self.layout_ctx.components.get_ticking(i) else { continue };
            let event = Event::Tick(dt);
            self.send_event_to_component(event, widget_id, state_id);
        }
    }

    fn init_new_components(&mut self) {
        while let Some((widget_id, state_id)) = self.layout_ctx.new_components.pop() {
            self.with_component(widget_id, state_id, |comp, elements, ctx| {
                comp.dyn_component.any_event(elements, ctx, Event::Mount);
            });
        }
    }

    // Return the state for each component back into the component registry
    fn return_state_and_component(self) {
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

    // fn display_error(&mut self, backend: &mut impl Backend) {
    //     let _tpl = "text 'you goofed up'";
    //     backend.render(self.layout_ctx.glyph_map);
    // }
}
