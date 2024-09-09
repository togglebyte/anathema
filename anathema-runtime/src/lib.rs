// ------------------
//   - Runtime -
//   1. Creating the initial widget tree
//   2. Runtime loop      <--------------------------------+
//    ^  2.1. Wait for messages                            |
//    |  2.2. Wait for events                              v
//    |  2.4. Was there events / messages / data changes? (no) (yes)
//    |                                                         ^
//    |                                                         |
//    |       +-------------------------------------------------+
//    |       |
//    |       V
//    |       1. Layout
//    |       2. Position
//    |       3. Draw
//    +------ 4. Run again
//
// -----------------------------------------------------------------------------

use std::fmt::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_default_widgets::register_default_widgets;
use anathema_state::{
    clear_all_changes, clear_all_futures, clear_all_subs, drain_changes, drain_futures, Changes, FutureValues, States,
};
use anathema_store::tree::root_node;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals, ToSourceKind};
use anathema_widgets::components::{
    AssociatedEvents, Component, ComponentId, ComponentKind, ComponentRegistry, Emitter, FocusQueue, UntypedContext,
    ViewMessage,
};
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Components, DirtyWidgets, EvalContext,
    Factory, FloatingWidgets, Scope, WidgetKind, WidgetTree,
};
use events::{EventCtx, EventHandler};
use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tree::Tree;

pub use self::events::{GlobalContext, GlobalEvents};
pub use crate::error::{Error, Result};

static REBUILD: AtomicBool = AtomicBool::new(false);

mod error;
mod events;
mod tree;

pub struct RuntimeBuilder<T, G> {
    document: Document,
    component_registry: ComponentRegistry,
    backend: T,
    factory: Factory,
    message_receiver: flume::Receiver<ViewMessage>,
    emitter: Emitter,
    global_events: G,
}

impl<T, G: GlobalEvents> RuntimeBuilder<T, G> {
    /// Registers a [Component] with the runtime.
    /// This returns a unique [ComponentId] that is used to send messages to the component.
    ///
    /// A component can only be used once in a template.
    /// If you want multiple instances, register the component as a prototype instead,
    /// see [RuntimeBuilder::register_prototype].
    pub fn register_component<C: Component + 'static>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        component: C,
        state: C::State,
    ) -> Result<ComponentId<C::Message>> {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.to_source_kind())?.into();
        self.component_registry.add_component(id, component, state);
        Ok(id.into())
    }

    pub fn global_events<U>(self, global_events: U) -> RuntimeBuilder<T, U> {
        RuntimeBuilder {
            document: self.document,
            component_registry: self.component_registry,
            backend: self.backend,
            factory: self.factory,
            message_receiver: self.message_receiver,
            emitter: self.emitter,
            global_events,
        }
    }

    /// Registers a [Component] as a prototype with the [Runtime],
    /// which allows for multiple instances of the component to exist the templates.
    pub fn register_prototype<FC, FS, C>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        proto: FC,
        state: FS,
    ) -> Result<()>
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> C::State,
        C: Component + 'static,
    {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.to_source_kind())?.into();
        self.component_registry.add_prototype(id, proto, state);
        Ok(())
    }

    /// Registers a [Component] with the runtime as long as the component and the associated state
    /// implements default.
    ///
    /// This is a shortcut for calling
    /// ```ignore
    /// runtime.register_component(
    ///     "name",
    ///     "template.aml",
    ///     TheComponent::default(),
    ///     TheComponent::State::default()
    /// );
    /// ```
    pub fn register_default<C>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
    ) -> Result<ComponentId<C::Message>>
    where
        C: Component + Default + 'static,
        C::State: Default,
    {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.to_source_kind())?.into();
        self.component_registry
            .add_component(id, C::default(), C::State::default());
        Ok(id.into())
    }

    /// Returns an [Emitter] to send messages to components
    pub fn emitter(&self) -> Emitter {
        self.emitter.clone()
    }

    fn set_watcher(&mut self) -> Result<RecommendedWatcher> {
        let paths = self
            .document
            .template_paths()
            .filter_map(|p| p.canonicalize().ok())
            .collect::<Vec<_>>();

        let mut watcher = recommended_watcher(move |event: std::result::Result<Event, _>| match event {
            Ok(event) => match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Remove(_) | notify::EventKind::Modify(_) => {
                    if paths.iter().any(|p| event.paths.contains(p)) {
                        REBUILD.store(true, Ordering::Relaxed);
                    }
                }
                notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => (),
            },
            Err(_err) => (),
        })?;

        for path in self.document.template_paths() {
            let path = path.canonicalize().unwrap();

            if let Some(parent) = path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }

        Ok(watcher)
    }

    /// Builds the [Runtime].
    /// Fails if compiling the [Document] or creating the file watcher fails.
    pub fn finish(mut self) -> Result<Runtime<T, G>>
    where
        T: Backend,
    {
        let (blueprint, globals) = self.document.compile()?;
        let watcher = match self.document.hot_reload {
            false => None,
            true => Some(self.set_watcher()?),
        };

        let (width, height) = self.backend.size().into();
        let constraints = Constraints::new(width as usize, height as usize);

        let inst = Runtime {
            _watcher: watcher,
            backend: self.backend,
            emitter: self.emitter,
            message_receiver: self.message_receiver,
            fps: 30,
            constraints,
            blueprint,
            factory: self.factory,
            future_values: FutureValues::empty(),

            changes: Changes::empty(),
            component_registry: self.component_registry,
            globals,
            document: self.document,
            viewport: Viewport::new((width, height)),
            floating_widgets: FloatingWidgets::empty(),
            components: Components::new(),
            dirty_widgets: DirtyWidgets::empty(),
            event_handler: EventHandler::new(self.global_events),
        };

        Ok(inst)
    }
}

/// A runtime for Anathema.
/// Needs a backend and a document.
/// ```
/// # use anathema_runtime::Runtime;
/// # use anathema_templates::Document;
/// # use anathema_backend::test::TestBackend;
/// # let backend = TestBackend::new((10, 10));
/// let document = Document::new("border");
/// let mut runtime = Runtime::builder(document, backend).finish().unwrap();
/// ```
pub struct Runtime<T, G> {
    pub fps: u16,

    _watcher: Option<RecommendedWatcher>,
    message_receiver: flume::Receiver<ViewMessage>,
    emitter: Emitter,
    blueprint: Blueprint,
    factory: Factory,
    globals: Globals,
    document: Document,

    // -----------------------------------------------------------------------------
    //   - Mut during runtime -
    // -----------------------------------------------------------------------------
    // * Event handling
    // * Layout
    backend: T,
    // * Event handling
    // * Layout (immutable)
    viewport: Viewport,
    // * Event handling
    event_handler: EventHandler<G>,
    // * Layout
    // * Event handling
    constraints: Constraints,
    // * Event handling
    components: Components,
    dirty_widgets: DirtyWidgets,
    // tab_indices: TabIndices,

    // -----------------------------------------------------------------------------
    //   - Mut during updates -
    // -----------------------------------------------------------------------------
    // * Changes
    changes: Changes,
    // * Futures
    future_values: FutureValues,
    // * Changes
    // * Futures
    component_registry: ComponentRegistry,
    // * Layout
    floating_widgets: FloatingWidgets,
}

impl<T> Runtime<T, ()>
where
    T: Backend,
{
    /// Creates a [RuntimeBuilder] based on the [Document] and the [Backend].
    pub fn builder(document: Document, backend: T) -> RuntimeBuilder<T, ()> {
        let mut factory = Factory::new();

        let (message_sender, message_receiver) = flume::unbounded();
        register_default_widgets(&mut factory);

        RuntimeBuilder {
            backend,
            document,
            component_registry: ComponentRegistry::new(),
            factory,
            emitter: message_sender.into(),
            message_receiver,
            global_events: (),
        }
    }
}

impl<T, G> Runtime<T, G>
where
    T: Backend,
    G: GlobalEvents,
{
    pub fn emitter(&self) -> Emitter {
        self.emitter.clone()
    }

    fn apply_futures<'bp>(
        &mut self,
        globals: &'bp Globals,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
    ) {
        drain_futures(&mut self.future_values);

        if self.future_values.is_empty() {
            return;
        }

        let mut scope = Scope::new();
        self.future_values.drain().rev().for_each(|sub| {
            scope.clear();
            let path = tree.path(sub);

            try_resolve_future_values(
                globals,
                &self.factory,
                &mut scope,
                states,
                &mut self.component_registry,
                sub,
                &path,
                tree,
                attribute_storage,
                &mut self.floating_widgets,
                &mut self.components,
            );
        });
    }

    fn apply_changes<'bp>(
        &mut self,
        globals: &'bp Globals,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
    ) {
        drain_changes(&mut self.changes);

        if self.changes.is_empty() {
            return;
        }

        let mut scope = Scope::new();
        self.changes.iter().for_each(|(sub, change)| {
            sub.iter().for_each(|sub| {
                scope.clear();
                let Some(path): Option<Box<_>> = tree.try_path_ref(sub).map(Into::into) else { return };

                update_tree(
                    globals,
                    &self.factory,
                    &mut scope,
                    states,
                    &mut self.component_registry,
                    change,
                    sub,
                    &path,
                    tree,
                    attribute_storage,
                    &mut self.floating_widgets,
                    &mut self.components,
                );
            });
        });
    }

    // Handles component messages for (ideally) at most half of a tick
    fn handle_messages<'bp>(
        &mut self,
        fps_now: Instant,
        sleep_micros: u128,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        assoc_events: &mut AssociatedEvents,
        focus_queue: &mut FocusQueue<'static>,
    ) -> Duration {
        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &mut self.document.strings,
        };

        let mut event_ctx = EventCtx {
            components: &mut self.components,
            dirty_widgets: &mut self.dirty_widgets,
            states,
            attribute_storage,
            assoc_events,
            focus_queue,
            context,
        };

        while let Ok(msg) = self.message_receiver.try_recv() {
            if let Some((widget_id, state_id)) = event_ctx
                .components
                .get_by_component_id(msg.recipient())
                .map(|e| (e.widget_id, e.state_id))
            {
                tree.with_component(widget_id, state_id, &mut event_ctx, |a, b| {
                    a.any_message(msg.payload(), b)
                });
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > sleep_micros / 2 {
                break;
            }
        }

        fps_now.elapsed()
    }

    /// Start the runtime
    pub fn run(&mut self) {
        self.backend.finalize();
        loop {
            match self.internal_run() {
                Ok(()) => (),
                Err(Error::Stop) => return,
                Err(err) => self.show_error(err),
            }
        }
    }

    // 1 - Tries to build the tree
    // 2 - Selects the first [Component] and calls [Component::on_focus] on it
    // 3 - Repeatedly calls [Self::tick] until [REBUILD] is set to true or an error occurs. Using the [Error::Stop] breaks the main loop.
    // 4 - Resets using [Self::reset]
    // 5 - Recursively calls [Self::internal_run].
    // TODO: We should move this into a loop in [Self::run].
    fn internal_run(&mut self) -> Result<()> {
        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut assoc_events = AssociatedEvents::new();
        let mut focus_queue = FocusQueue::new();

        let mut states = States::new();
        let mut scope = Scope::new();
        let globals = self.globals.take();

        let mut ctx = EvalContext::new(
            &globals,
            &self.factory,
            &mut scope,
            &mut states,
            &mut self.component_registry,
            &mut attribute_storage,
            &mut self.floating_widgets,
            &mut self.components,
        );

        let blueprint = self.blueprint.clone();

        // First build the tree
        let res = eval_blueprint(&blueprint, &mut ctx, root_node(), &mut tree);

        match res {
            Ok(_) => (),
            Err(err) => {
                match self.reset(tree, &mut states) {
                    Ok(()) => (),
                    Err(err) => return Err(err),
                }
                return Err(err.into());
            }
        }

        let mut dt = Instant::now();

        // Initial layout, position and paint
        WidgetCycle::new(
            &mut self.backend,
            &mut tree,
            self.constraints,
            &attribute_storage,
            &self.floating_widgets,
            self.viewport,
        )
        .run();
        self.backend.render();
        self.backend.clear();

        // Try to set focus on the first available component
        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &self.document.strings,
        };

        let mut event_ctx = EventCtx {
            components: &mut self.components,
            dirty_widgets: &mut self.dirty_widgets,
            states: &mut states,
            attribute_storage: &mut attribute_storage,
            assoc_events: &mut assoc_events,
            context,
            focus_queue: &mut focus_queue,
        };

        self.event_handler.set_initial_focus(&mut tree, &mut event_ctx);

        loop {
            self.tick(
                fps_now,
                &mut dt,
                sleep_micros,
                &mut tree,
                &mut states,
                &mut attribute_storage,
                &globals,
                &mut assoc_events,
                &mut focus_queue,
            )?;

            if REBUILD.swap(false, Ordering::Relaxed) {
                break;
            }

            fps_now = Instant::now();
        }

        self.reset(tree, &mut states)
    }

    pub fn show_error(&mut self, err: Error) {
        let tpl = "
            align [alignment: 'centre']
                border [background: 'red']
                    vstack
                        @errors
        "
        .to_string();

        let errors = err.to_string().lines().fold(String::new(), |mut s, line| {
            let _ = writeln!(&mut s, "text [foreground: 'black'] '{line}'");
            s
        });

        let mut document = Document::new(tpl);
        let _component_id = document.add_component("errors", errors.to_template());
        let (blueprint, globals) = document.compile().expect("the error template can't fail");
        self.blueprint = blueprint;
        self.globals = globals;
    }

    // Resets the Runtime:
    // * Reloads all components
    // * Moves all the components from the tree back to the registry.
    // * Recompiles the document
    fn reset(&mut self, tree: WidgetTree<'_>, states: &mut States) -> Result<()> {
        clear_all_futures();
        clear_all_changes();
        clear_all_subs();

        self.components = Components::new();
        self.floating_widgets = FloatingWidgets::empty();

        // The only way we can get here is if we break the loop
        // as a result of the hot_reload triggering or when building the first tree fails.
        self.document.reload_templates()?;

        // Move all components from the tree back to the registry.
        for (_, widget) in tree.values().into_iter() {
            let WidgetKind::Component(comp) = widget else { continue };
            let ComponentKind::Instance = comp.kind else { continue };
            let state = states.remove(comp.state_id);
            self.component_registry
                .return_component(comp.component_id, comp.dyn_component, state);
        }

        let (blueprint, globals) = self.document.compile()?;
        self.blueprint = blueprint;
        self.globals = globals;

        Ok(())
    }

    fn tick<'bp>(
        &mut self,
        fps_now: Instant,
        dt: &mut Instant,
        sleep_micros: u128,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        globals: &'bp Globals,
        assoc_events: &mut AssociatedEvents,
        focus_queue: &mut FocusQueue<'static>,
    ) -> Result<()> {
        // Clear the text buffer
        // self.string_storage.clear();

        // Pull and keep consuming events while there are events present in the queue.
        let poll_duration = self.handle_messages(
            fps_now,
            sleep_micros,
            tree,
            states,
            attribute_storage,
            assoc_events,
            focus_queue,
        );

        // Call the `tick` function on all components
        self.tick_components(tree, states, attribute_storage, dt.elapsed(), assoc_events, focus_queue);

        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &self.document.strings,
        };

        let mut event_ctx = EventCtx {
            components: &mut self.components,
            dirty_widgets: &mut self.dirty_widgets,
            states,
            attribute_storage,
            assoc_events,
            context,
            focus_queue,
        };

        self.event_handler.handle(
            poll_duration,
            fps_now,
            sleep_micros,
            &mut self.backend,
            &mut self.viewport,
            tree,
            &mut self.constraints,
            &mut event_ctx,
        )?;

        *dt = Instant::now();

        self.apply_futures(globals, tree, states, attribute_storage);

        self.apply_changes(globals, tree, states, attribute_storage);

        // -----------------------------------------------------------------------------
        //   - Update dirty widgets -
        //   Mark dirty widgets for redraw, along with their parents
        // -----------------------------------------------------------------------------
        self.dirty_widgets.apply(tree);

        // Cleanup removed attributes from widgets.
        for key in tree.drain_removed() {
            attribute_storage.try_remove(key);
            self.floating_widgets.try_remove(key);
            // TODO: this function is rubbish and has to be rewritten
            self.components.dodgy_remove(key);
        }

        // -----------------------------------------------------------------------------
        //   - Layout, position and paint -
        // -----------------------------------------------------------------------------
        let needs_reflow = !self.changes.is_empty() || !self.dirty_widgets.is_empty();
        if needs_reflow {
            let mut cycle = WidgetCycle::new(
                &mut self.backend,
                tree,
                self.constraints,
                attribute_storage,
                &self.floating_widgets,
                self.viewport,
            );
            cycle.run();

            self.backend.render();
            self.backend.clear();
            self.changes.clear();
            self.dirty_widgets.clear();
        }

        let sleep = sleep_micros.saturating_sub(fps_now.elapsed().as_micros()) as u64;
        if sleep > 0 {
            std::thread::sleep(Duration::from_micros(sleep));
        }

        Ok(())
    }

    fn tick_components<'bp>(
        &mut self,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        dt: Duration,
        assoc_events: &mut AssociatedEvents,
        focus_queue: &mut FocusQueue<'static>,
    ) {
        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &self.document.strings,
        };

        for i in 0..self.components.len() {
            let (widget_id, state_id) = self
                .components
                .get(i)
                .expect("the components can not change as a result of this step");

            let mut event_ctx = EventCtx {
                components: &mut self.components,
                dirty_widgets: &mut self.dirty_widgets,
                states,
                attribute_storage,
                assoc_events,
                focus_queue,
                context,
            };

            tree.with_component(widget_id, state_id, &mut event_ctx, |a, b| a.any_tick(b, dt));
        }
    }
}
