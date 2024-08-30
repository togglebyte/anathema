// -----------------------------------------------------------------------------
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

use anathema_backend::Backend;
use anathema_default_widgets::register_default_widgets;
use anathema_geometry::Pos;
use anathema_state::{
    clear_all_changes, clear_all_futures, clear_all_subs, drain_changes, drain_futures, Changes, FutureValues, States,
};
use anathema_store::tree::{root_node, AsNodePath};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals, ToSourceKind};
use anathema_widgets::components::{
    AssociatedEvents, Component, ComponentId, ComponentKind, ComponentRegistry, Emitter, UntypedContext, ViewMessage,
};
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Components, EvalContext, Factory,
    FloatingWidgets, Scope, WidgetKind, WidgetTree,
};
use events::{EventCtx, EventHandler};
use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tree::Tree;

pub use crate::error::{Error, Result};

static REBUILD: AtomicBool = AtomicBool::new(false);

mod error;
mod events;
mod tree;

pub struct RuntimeBuilder<T> {
    document: Document,
    component_registry: ComponentRegistry,
    backend: T,
    factory: Factory,
    message_receiver: flume::Receiver<ViewMessage>,
    emitter: Emitter,
}

impl<T> RuntimeBuilder<T> {
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

    pub fn finish(mut self) -> Result<Runtime<T>>
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
            string_storage: StringStorage::new(),
            viewport: Viewport::new((width, height)),
            floating_widgets: FloatingWidgets::empty(),
            components: Components::new(),
            event_handler: EventHandler,
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
/// let mut runtime = Runtime::new(document, backend).finish().unwrap();
/// ```
pub struct Runtime<T> {
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
    event_handler: EventHandler,
    // * Layout
    string_storage: StringStorage,
    // * Event handling
    constraints: Constraints,
    // * Event handling
    components: Components,
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

impl<T> Runtime<T>
where
    T: Backend,
{
    #[deprecated(note = "use the `builder` function instead")]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(document: Document, backend: T) -> RuntimeBuilder<T> {
        Self::builder(document, backend)
    }

    pub fn builder(document: Document, backend: T) -> RuntimeBuilder<T> {
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
        }
    }

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
        self.changes.drain().rev().for_each(|(sub, change)| {
            sub.iter().for_each(|sub| {
                scope.clear();
                let Some(path): Option<Box<_>> = tree.try_path_ref(sub).map(Into::into) else { return };

                update_tree(
                    globals,
                    &self.factory,
                    &mut scope,
                    states,
                    &mut self.component_registry,
                    &change,
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

    fn handle_messages<'bp>(
        &mut self,
        fps_now: Instant,
        sleep_micros: u128,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        assoc_events: &mut AssociatedEvents,
    ) -> Duration {
        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &mut self.document.strings,
        };

        let mut event_ctx = EventCtx {
            components: &mut self.components,
            states,
            attribute_storage,
            assoc_events,
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

    pub fn run(&mut self) {
        self.backend.finalize();
        match self.internal_run() {
            Ok(()) => (),
            Err(Error::Stop) => (),
            Err(err) => {
                self.show_error(err);
                self.run();
            }
        }
    }

    fn internal_run(&mut self) -> Result<()> {
        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut assoc_events = AssociatedEvents::new();

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

        // // Select the first widget
        // if let Some((widget_id, state_id)) = self.components.current() {
        //     tree.with_value_mut(widget_id, |path, widget, tree| {
        //         let WidgetKind::Component(component) = widget else { return };
        //         let state = states.get_mut(state_id);

        //         let parent = component
        //             .parent
        //             .and_then(|parent| self.components.get_by_component_id(parent))
        //             .map(|parent| parent.component_id.into());

        //         let Some((node, values)) = tree.get_node_by_path(path) else { return };
        //         let elements = Elements::new(node.children(), values, &mut attribute_storage);

        //         let component_ctx = ComponentContext::new(state_id, component.parent, component.assoc_functions);

        //         component
        //             .dyn_component
        //             .any_focus(state, elements, context, component_ctx);
        //     });
        // }

        let mut dt = Instant::now();
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
            )?;

            if REBUILD.swap(false, Ordering::Relaxed) {
                break;
            }

            fps_now = Instant::now();
        }

        self.reset(tree, &mut states)?;
        self.internal_run()
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

    fn reset(&mut self, tree: WidgetTree<'_>, states: &mut States) -> Result<()> {
        clear_all_futures();
        clear_all_changes();
        clear_all_subs();

        self.components = Components::new();
        self.floating_widgets = FloatingWidgets::empty();
        self.string_storage = StringStorage::new();

        // The only way we can get here is if we break the loop
        // as a result of the hot_reload triggering.
        self.document.reload_templates()?;

        // move all components from the tree back to the registry.
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

    pub fn tick<'bp>(
        &mut self,
        fps_now: Instant,
        dt: &mut Instant,
        sleep_micros: u128,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        globals: &'bp Globals,
        assoc_events: &mut AssociatedEvents,
    ) -> Result<()> {
        // Pull and keep consuming events while there are events present
        // in the queu. The time used to pull events should be subtracted
        // from the poll duration of self.events.poll
        let poll_duration = self.handle_messages(fps_now, sleep_micros, tree, states, attribute_storage, assoc_events);

        // Clear the text buffer
        self.string_storage.clear();

        // Call the `tick` function on all components
        self.tick_components(tree, states, attribute_storage, dt.elapsed(), assoc_events);

        let context = UntypedContext {
            emitter: &self.emitter,
            viewport: self.viewport,
            strings: &self.document.strings,
        };

        let mut event_ctx = EventCtx {
            components: &mut self.components,
            states,
            attribute_storage,
            assoc_events,
            context,
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

        // TODO
        // Instead of draining the changes when applying
        // the changes, we can keep the changes and use them in the
        // subsequent update / position / paint sequence
        //
        // Store the size and constraint on a widget
        //
        // * If the widget changes but the size remains the same then
        //   there is no reason to perform a layout on the entire tree,
        //   and only the widget it self needs to be re-painted
        //
        // Q) What about floating widgets?

        self.apply_changes(globals, tree, states, attribute_storage);

        // Cleanup removed attributes from widgets.
        // Not all widgets has attributes, only `Element`s.
        for key in tree.drain_removed() {
            attribute_storage.try_remove(key);
            self.floating_widgets.try_remove(key);
            // TODO: this function is rubbish and has to be rewritten
            self.components.dodgy_remove(key);
        }

        // -----------------------------------------------------------------------------
        //   - Layout, position and paint -
        // -----------------------------------------------------------------------------
        let mut filter = LayoutFilter::new(true, attribute_storage);
        tree.for_each(&mut filter).first(&mut |widget, children, values| {
            // Layout
            // TODO: once the text buffer can be read-only for the paint
            //       the context can be made outside of this closure.
            //
            //       That doesn't have as much of an impact here
            //       as it will do when dealing with the floating widgets
            let mut layout_ctx = LayoutCtx::new(self.string_storage.new_session(), attribute_storage, &self.viewport);
            layout_widget(widget, children, values, self.constraints, &mut layout_ctx, true);

            // Position
            position_widget(Pos::ZERO, widget, children, values, attribute_storage, true);

            // Paint
            let mut string_session = self.string_storage.new_session();
            self.backend
                .paint(widget, children, values, &mut string_session, attribute_storage, true);
        });

        // Floating widgets
        for widget_id in self.floating_widgets.iter() {
            // Find the parent widget and get the position
            // If no parent element is found assume Pos::ZERO
            let mut parent = tree.path_ref(*widget_id).parent();
            let (pos, constraints) = loop {
                match parent {
                    None => break (Pos::ZERO, self.constraints),
                    Some(p) => match tree.get_ref_by_path(p) {
                        Some(WidgetKind::Element(el)) => break (el.get_pos(), Constraints::from(el.size())),
                        _ => parent = p.parent(),
                    },
                }
            };

            tree.with_nodes_and_values(*widget_id, |widget, children, values| {
                let WidgetKind::Element(el) = widget else { unreachable!("this is always a floating widget") };
                let mut layout_ctx =
                    LayoutCtx::new(self.string_storage.new_session(), attribute_storage, &self.viewport);

                layout_widget(el, children, values, constraints, &mut layout_ctx, true);

                // Position
                position_widget(pos, el, children, values, attribute_storage, true);

                // Paint
                let mut string_session = self.string_storage.new_session();
                self.backend
                    .paint(el, children, values, &mut string_session, attribute_storage, true);
            });
        }

        self.backend.render();
        self.backend.clear();

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
                states,
                attribute_storage,
                assoc_events,
                context,
            };

            tree.with_component(widget_id, state_id, &mut event_ctx, |a, b| a.any_tick(b, dt));
        }
    }
}
