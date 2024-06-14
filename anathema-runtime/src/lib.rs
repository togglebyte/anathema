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

use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_default_widgets::register_default_widgets;
use anathema_geometry::Pos;
use anathema_state::{drain_changes, drain_futures, Changes, FutureValues, State, States};
use anathema_store::tree::{AsNodePath, NodePath};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::{Component, ComponentId, ComponentRegistry};
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Elements, EvalContext, Factory,
    FloatingWidgets, Scope, Widget, WidgetKind, WidgetTree,
};
use components::Components;
use events::EventHandler;

pub use crate::error::Result;
pub use crate::messages::{Emitter, ViewMessage};

mod components;
mod error;
mod events;
mod messages;

pub struct RuntimeBuilder<T> {
    document: Document,
    component_registry: ComponentRegistry,
    backend: T,
}

impl<T> RuntimeBuilder<T> {
    pub fn register_component<S: 'static + State>(
        &mut self,
        ident: impl Into<String>,
        template: impl Into<String>,
        component: impl Component + 'static,
        state: S,
    ) -> ComponentId {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.into()).into();
        self.component_registry.add_component(id, component, state);
        id
    }

    pub fn register_prototype<FC, FS, C, S>(
        &mut self,
        ident: impl Into<String>,
        template: impl Into<String>,
        proto: FC,
        state: FS,
    ) where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.into());
        self.component_registry.add_prototype(id.into(), proto, state);
    }

    pub fn finish(self) -> Result<Runtime<T>>
    where
        T: Backend,
    {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let (bp, globals) = self.document.compile()?;

        let (tx, rx) = flume::unbounded();

        let (width, height) = self.backend.size().into();
        let constraints = Constraints::new(width as usize, height as usize);

        let inst = Runtime {
            backend: self.backend,
            message_sender: tx,
            message_receiver: rx,
            fps: 30,
            constraints,
            bp,
            factory,
            future_values: FutureValues::empty(),
            changes: Changes::empty(),
            components: Components::new(),
            component_registry: self.component_registry,
            globals,
            string_storage: StringStorage::new(),
            viewport: Viewport::new((width, height)),
            floating_widgets: FloatingWidgets::empty(),
            event_handler: EventHandler::new(),
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
/// let mut runtime = Runtime::new(document, backend).unwrap();
/// ```
pub struct Runtime<T> {
    pub fps: u16,

    message_receiver: flume::Receiver<ViewMessage>,
    message_sender: flume::Sender<ViewMessage>,
    bp: Blueprint,
    factory: Factory,
    globals: Globals,

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
    pub fn new(document: Document, backend: T) -> RuntimeBuilder<T> {
        RuntimeBuilder {
            backend,
            document,
            component_registry: ComponentRegistry::new(),
        }
    }

    pub fn register_component<S: 'static + State>(
        &mut self,
        id: impl Into<ComponentId>,
        component: impl Component<State = S> + 'static,
        state: S,
    ) {
        self.component_registry.add_component(id.into(), component, state);
    }

    pub fn register_prototype<FC, FS, C, S>(&mut self, id: impl Into<ComponentId>, proto: FC, state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        self.component_registry.add_prototype(id.into(), proto, state);
    }

    pub fn emitter(&self) -> Emitter {
        Emitter(self.message_sender.clone())
    }

    pub fn register_default_widget<W: 'static + Widget + Default>(&mut self, ident: &str) {
        self.factory.register_default::<W>(ident);
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
            let path = tree.path(sub).clone();

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
                let Some(path) = tree.try_path(sub).cloned() else { return };

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
    ) -> Duration {
        while let Ok(msg) = self.message_receiver.try_recv() {
            if let Some(entry) = self.components.dumb_fetch(msg.recipient) {
                tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                    let WidgetKind::Component(component) = widget else { return };
                    let state = entry.state_id.and_then(|id| states.get_mut(id));
                    let Some((node, values)) = tree.get_node_by_path(path) else { return };
                    let elements = Elements::new(node.children(), values, attribute_storage);
                    component.component.any_message(msg.payload, state, elements);
                });
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > sleep_micros / 2 {
                break;
            }
        }

        fps_now.elapsed()
    }

    pub fn run(&mut self) -> Result<()> {
        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();

        let mut states = States::new();
        let mut scope = Scope::new();
        let globals = self.globals.clone();
        let mut ctx = EvalContext::new(
            &globals,
            &self.factory,
            &mut scope,
            &mut states,
            &mut self.component_registry,
            &mut attribute_storage,
            &mut self.floating_widgets,
        );

        let bp = self.bp.clone();
        // First build the tree
        eval_blueprint(&bp, &mut ctx, &NodePath::root(), &mut tree);

        // ... then the tab indices
        tree.apply_visitor(&mut self.components);

        // Select the first widget
        if let Some(entry) = self.components.current() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, &mut attribute_storage);
                component.component.any_focus(state, elements);
            });
        }

        loop {
            self.tick(
                fps_now,
                sleep_micros,
                &mut tree,
                &mut states,
                &mut attribute_storage,
                &globals,
            )?;
            fps_now = Instant::now();
        }
    }

    pub fn tick<'bp>(
        &mut self,
        fps_now: Instant,
        sleep_micros: u128,
        tree: &mut WidgetTree<'bp>,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        globals: &'bp Globals,
    ) -> Result<()> {
        // Pull and keep consuming events while there are events present
        // in the queu. The time used to pull events should be subtracted
        // from the poll duration of self.events.poll
        let poll_duration = self.handle_messages(fps_now, sleep_micros, tree, states, attribute_storage);

        // Clear the text buffer
        self.string_storage.clear();

        self.event_handler.handle(
            poll_duration,
            fps_now,
            sleep_micros,
            &mut self.backend,
            &mut self.viewport,
            tree,
            &mut self.components,
            states,
            attribute_storage,
            &mut self.constraints,
        )?;

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
            let mut parent = tree.path(*widget_id).pop();
            let (pos, constraints) = loop {
                match parent {
                    None => break (Pos::ZERO, self.constraints),
                    Some(p) => match tree.get_ref_by_path(p) {
                        Some(WidgetKind::Element(el)) => break (el.get_pos(), Constraints::from(el.size())),
                        _ => parent = p.pop(),
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

        // Cleanup removed attributes from widgets.
        // Not all widgets has attributes, only `Element`s.
        for key in tree.drain_removed() {
            attribute_storage.try_remove(key);
            self.floating_widgets.try_remove(key);
        }

        let sleep = sleep_micros.saturating_sub(fps_now.elapsed().as_micros()) as u64;
        if sleep > 0 {
            std::thread::sleep(Duration::from_micros(sleep));
        }

        Ok(())
    }
}

// struct Futures {
//     future_values: FutureValues
// }

// impl Futures {
//     fn new() -> Self {
//         Self {
//             future_values: FutureValues::empty(),
//         }
//     }

//     fn update(&mut self) {
//         drain_futures(&mut self.future_values);
//         let mut scope = Scope::new();
//         self.future_values.drain().rev().for_each(|sub| {
//             scope.clear();
//             let path = tree.path(sub).clone();

//             try_resolve_future_values(
//                 globals,
//                 &self.factory,
//                 &mut scope,
//                 states,
//                 &mut self.components,
//                 sub,
//                 &path,
//                 tree,
//                 attribute_storage,
//                 &mut self.floating_widgets,
//             );
//         });
//     }
// }
