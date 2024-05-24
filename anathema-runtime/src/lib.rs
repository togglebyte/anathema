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
use anathema_geometry::{Pos, Size};
use anathema_state::{drain_changes, drain_futures, Changes, FutureValues, State, States};
use anathema_store::tree::{AsNodePath, NodePath};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals};
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::components::{Component, ComponentId, ComponentRegistry};
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Elements, EvalContext, Factory,
    FloatingWidgets, Scope, Widget, WidgetKind, WidgetTree,
};
use error::Error;
use flume::Receiver;
use tabindex::TabIndex;

pub use crate::error::Result;
pub use crate::messages::{Emitter, ViewMessage};

mod error;
mod messages;
mod tabindex;

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
    backend: T,

    message_receiver: flume::Receiver<ViewMessage>,
    message_sender: flume::Sender<ViewMessage>,
    bp: Blueprint,
    constraints: Constraints,
    factory: Factory,
    tab_indices: TabIndex,
    future_values: FutureValues,
    changes: Changes,
    components: ComponentRegistry,
    globals: Globals,
    string_storage: StringStorage,
    viewport: Viewport,
    floating_widgets: FloatingWidgets,
}

impl<T> Runtime<T>
where
    T: Backend,
{
    pub fn new(doc: Document, backend: T) -> Result<Self> {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let (bp, globals) = doc.compile()?;

        let (tx, rx) = flume::unbounded();

        let (width, height) = backend.size().into();
        let constraints = Constraints::new(width as usize, height as usize);

        let inst = Self {
            backend,
            message_sender: tx,
            message_receiver: rx,
            fps: 30,
            constraints,
            bp,
            factory,
            future_values: FutureValues::empty(),
            changes: Changes::empty(),
            tab_indices: TabIndex::new(),
            components: ComponentRegistry::new(),
            globals,
            string_storage: StringStorage::new(),
            viewport: Viewport::new((width, height)),
            floating_widgets: FloatingWidgets::empty(),
        };

        Ok(inst)
    }

    pub fn register_component<S: 'static + State>(
        &mut self,
        id: impl Into<ComponentId>,
        component: impl Component + 'static,
        state: S,
    ) {
        self.components.add_component(id.into(), component, state);
    }

    pub fn register_prototype<FC, FS, C, S>(&mut self, id: impl Into<ComponentId>, proto: FC, state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        self.components.add_prototype(id.into(), proto, state);
    }

    pub fn emitter(&self) -> Emitter {
        Emitter(self.message_sender.clone())
    }

    pub fn register_default_widget<W: 'static + Widget + Default>(&mut self, ident: &str) {
        self.factory.register_default::<W>(ident);
    }

    pub fn run(&mut self) -> Result<()> {
        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();

        let mut states = States::new();
        let mut scope = Scope::new();
        let globals = self.globals.clone();
        let mut ctx = EvalContext::new(
            &globals,
            &self.factory,
            &mut scope,
            &mut states,
            &mut self.components,
            &mut attribute_storage,
            &mut floating_widgets,
        );

        let bp = self.bp.clone();
        // First build the tree
        eval_blueprint(&bp, &mut ctx, &NodePath::root(), &mut tree);

        let size = self.backend.size();

        // ... then the tab indices
        tree.apply_visitor(&mut self.tab_indices);

        // Select the first widget
        if let Some(entry) = self.tab_indices.current() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, &mut attribute_storage);
                component.component.any_focus(state, elements);
            });
        }

        'run: loop {
            self.tick(fps_now, sleep_micros, &mut tree, &mut states, &mut attribute_storage, &globals)?;
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
        let poll_duration = handle_messages(
            &self.message_receiver,
            &self.tab_indices,
            fps_now,
            sleep_micros,
            tree,
            states,
            attribute_storage,
        );

        // Clear the text buffer
        self.string_storage.clear();

        while let Some(event) = self.backend.next_event(poll_duration) {
            let event = global_event(
                &mut self.backend,
                &mut self.tab_indices,
                event,
                tree,
                states,
                attribute_storage,
            );

            // Ignore mouse events, as they are handled by global event
            if !event.is_mouse_event() {
                if let Some(entry) = self.tab_indices.current() {
                    tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                        let WidgetKind::Component(component) = widget else { return };
                        let state = entry.state_id.and_then(|id| states.get_mut(id));
                        let Some((node, values)) = tree.get_node_by_path(path) else { return };
                        let elements = Elements::new(node.children(), values, attribute_storage);
                        component.component.any_event(event, state, elements);
                    });
                }
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > sleep_micros {
                break;
            }

            match event {
                Event::Resize(width, height) => {
                    let size = Size::from((width, height));
                    self.backend.resize(size);
                    self.viewport.resize(size);
                    self.constraints.set_max_width(size.width);
                    self.constraints.set_max_height(size.height);
                }
                Event::Blur => (),
                Event::Focus => (),
                Event::Stop => return Err(Error::Stop),
                _ => {}
            }
        }

        apply_futures(
            globals,
            &mut self.future_values,
            &self.factory,
            tree,
            states,
            &mut self.components,
            attribute_storage,
            &mut self.floating_widgets,
        );

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

        // panic!("see this TODO and the one about updating widgets");

        apply_changes(
            globals,
            &mut self.changes,
            &self.factory,
            tree,
            states,
            &mut self.components,
            attribute_storage,
            &mut self.floating_widgets,
        );

        // -----------------------------------------------------------------------------
        //   - Layout, position and paint -
        // -----------------------------------------------------------------------------
        let mut filter = LayoutFilter::new(true, &attribute_storage);
        tree.for_each(&mut filter).first(&mut |widget, children, values| {
            // Layout
            // TODO: once the text buffer can be read-only for the paint
            //       the context can be made outside of this closure.
            //
            //       That doesn't have as much of an impact here
            //       as it will do when dealing with the floating widgets
            let mut layout_ctx = LayoutCtx::new(self.string_storage.new_session(), &attribute_storage, &self.viewport);
            layout_widget(widget, children, values, self.constraints, &mut layout_ctx, true);

            // Position
            position_widget(Pos::ZERO, widget, children, values, &attribute_storage, true);

            // Paint
            let mut string_session = self.string_storage.new_session();
            self.backend.paint(widget, children, values, &mut string_session, &attribute_storage, true);
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
                let mut layout_ctx = LayoutCtx::new(self.string_storage.new_session(), &attribute_storage, &self.viewport);

                layout_widget(el, children, values, constraints, &mut layout_ctx, true);

                // Position
                position_widget(pos, el, children, values, &attribute_storage, true);

                // Paint
                let mut string_session = self.string_storage.new_session();
                self.backend.paint(el, children, values, &mut string_session, &attribute_storage, true);
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

        // TODO unreachable
        Ok(())
    }
}

fn global_event<'bp, T: Backend>(
    backend: &mut T,
    tab_indices: &mut TabIndex,
    event: Event,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    attribute_storage: &mut AttributeStorage<'bp>,
) -> Event {
    // -----------------------------------------------------------------------------
    //   - Ctrl-c to quite -
    //   This should be on by default.
    //   Give it a good name
    // -----------------------------------------------------------------------------
    if backend.quit_test(event) {
        return Event::Stop;
    }

    // -----------------------------------------------------------------------------
    //   - Handle tabbing between components -
    // -----------------------------------------------------------------------------
    if let Event::Key(KeyEvent { code, .. }) = event {
        let prev = match code {
            KeyCode::Tab => tab_indices.next(),
            KeyCode::BackTab => tab_indices.prev(),
            _ => return event,
        };

        if let Some(entry) = prev {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_blur(state, elements);
            });
        }

        if let Some(entry) = tab_indices.current() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_focus(state, elements);
            });
        }
    }

    // Mouse events are global
    if let Event::Mouse(_) = event {
        for entry in tab_indices.iter() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                let _ = component.component.any_event(event, state, elements);
            });
        }
    }

    event
}

fn handle_messages<'bp>(
    message_receiver: &Receiver<ViewMessage>,
    tab_indices: &TabIndex,
    fps_now: Instant,
    sleep_micros: u128,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    attribute_storage: &mut AttributeStorage<'bp>,
) -> Duration {
    while let Ok(msg) = message_receiver.try_recv() {
        if let Some(entry) = tab_indices.dumb_fetch(msg.recipient) {
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

fn apply_futures<'bp>(
    globals: &'bp Globals,
    future_values: &mut FutureValues,
    factory: &Factory,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    drain_futures(future_values);

    let mut scope = Scope::new();
    future_values.drain().rev().for_each(|sub| {
        scope.clear();
        let path = tree.path(sub).clone();

        try_resolve_future_values(
            globals,
            factory,
            &mut scope,
            states,
            components,
            sub,
            &path,
            tree,
            attribute_storage,
            floating_widgets,
        );
    });
}

fn apply_changes<'bp>(
    globals: &'bp Globals,
    changes: &mut Changes,
    factory: &Factory,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    drain_changes(changes);

    if changes.is_empty() {
        return;
    }

    let mut scope = Scope::new();
    changes.drain().rev().for_each(|(sub, change)| {
        sub.iter().for_each(|sub| {
            scope.clear();
            let Some(path) = tree.try_path(sub).cloned() else { return };

            update_tree(
                globals,
                factory,
                &mut scope,
                states,
                components,
                &change,
                sub,
                &path,
                tree,
                attribute_storage,
                floating_widgets,
            );
        });
    });
}
