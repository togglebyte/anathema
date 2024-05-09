// -----------------------------------------------------------------------------
//   - Runtime -
//   1. Creating the initial widget tree
//   2. Runtime loop      <---------------------------------+
//    ^  2.1. Wait for messages                             |
//    |  2.2. Wait for events                               |
//    |  2.4. Was there events / messages / data changes? (no) (yes)
//    |                                                           |
//    |                                                           |
//    |        +--------------------------------------------------+
//    |        |
//    |        V
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
use anathema_templates::Document;
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::components::{Component, ComponentId, ComponentRegistry};
use anathema_widgets::layout::text::StringStorage;
use anathema_widgets::layout::{layout_widget, position_widget, Constraints, LayoutCtx, LayoutFilter, Viewport};
use anathema_widgets::{
    eval_blueprint, try_resolve_future_values, update_tree, AttributeStorage, Elements, EvalContext, Factory,
    FloatingWidgets, Scope, Widget, WidgetKind, WidgetTree,
};
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
}

impl<T> Runtime<T>
where
    T: Backend,
{
    pub fn new(doc: Document, backend: T) -> Result<Self> {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let bp = doc.compile()?.remove(0);

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
        FS: 'static + Fn() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        self.components.add_prototype(id.into(), proto, state);
    }

    pub fn emitter(&self) -> Emitter {
        Emitter(self.message_sender.clone())
    }

    pub fn run(mut self) -> Result<()> {
        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;
        let mut tree = WidgetTree::empty();
        let mut attribute_storage = AttributeStorage::empty();
        let mut floating_widgets = FloatingWidgets::empty();

        let components = &mut self.components;
        let mut states = States::new();
        let mut scope = Scope::new();
        let mut ctx = EvalContext::new(
            &self.factory,
            &mut scope,
            &mut states,
            components,
            &mut attribute_storage,
            &mut floating_widgets,
        );
        let bp = self.bp.clone(); // TODO ewwww

        // First build the tree
        eval_blueprint(&bp, &mut ctx, &NodePath::root(), &mut tree);

        let Self {
            mut backend,
            message_receiver,
            factory,
            mut future_values,
            mut changes,
            mut tab_indices,
            mut components,
            ..
        } = self;

        let mut string_storage = StringStorage::new();

        let size = backend.size();
        let mut viewport = Viewport::new(size);

        // ... then the tab indices
        tree.apply_visitor(&mut tab_indices);

        // Select the first widget
        if let Some(entry) = tab_indices.current() {
            if let Some(WidgetKind::Component(component)) = tree.get_mut_by_id(entry.widget_id) {
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_focus(state);
            }
        }

        'run: loop {
            // Pull and keep consuming events while there are events present
            // in the queu. The time used to pull events should be subtracted
            // from the poll duration of self.events.poll
            let poll_duration = handle_messages(
                &message_receiver,
                &tab_indices,
                fps_now,
                sleep_micros,
                &mut tree,
                &mut states,
            );

            // Clear the text buffer
            string_storage.clear();

            apply_futures(
                &mut future_values,
                &factory,
                &mut tree,
                &mut states,
                &mut components,
                &mut attribute_storage,
                &mut floating_widgets,
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
                &mut changes,
                &factory,
                &mut tree,
                &mut states,
                &mut components,
                &mut attribute_storage,
                &mut floating_widgets,
            );

            while let Some(event) = backend.next_event(poll_duration) {
                let event = global_event(&mut backend, &mut tab_indices, event, &mut tree, &mut states);

                if let Some(entry) = tab_indices.current() {
                    tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                        let WidgetKind::Component(component) = widget else { return };
                        let Some((node, values)) = tree.get_node_by_path(path) else { return };
                        let state = entry.state_id.and_then(|id| states.get_mut(id));
                        let widgets = Elements::new(node.children(), values, &mut attribute_storage);
                        component.component.any_event(event, state, widgets);
                    });
                }

                // Make sure event handling isn't holding up the rest of the event loop.
                if fps_now.elapsed().as_micros() > sleep_micros {
                    break;
                }

                match event {
                    Event::Resize(width, height) => {
                        let size = Size::from((width, height));
                        backend.resize(size);
                        viewport.resize(size);
                        self.constraints.set_max_width(size.width);
                        self.constraints.set_max_height(size.height);
                    }
                    Event::Blur => (),
                    Event::Focus => (),
                    Event::Stop => break 'run Ok(()),
                    _ => {}
                }

                if let Event::Stop = event {
                    break 'run Ok(());
                }
            }

            let mut filter = LayoutFilter::new(true);
            tree.for_each(&mut filter).first(&mut |widget, children, values| {
                // Layout
                // TODO: once the text buffer can be read-only for the paint
                //       the context can be made outside of this closure.
                //
                //       That doesn't have as much of an impact here
                //       as it will do when dealing with the floating widgets
                let mut layout_ctx = LayoutCtx::new(string_storage.new_session(), &attribute_storage, &viewport);
                layout_widget(widget, children, values, self.constraints, &mut layout_ctx, true);

                // Position
                position_widget(Pos::ZERO, widget, children, values, &attribute_storage, true);

                // Paint
                let mut string_session = string_storage.new_session();
                backend.paint(widget, children, values, &mut string_session, &attribute_storage, true);
            });

            // Floating widgets
            for widget_id in floating_widgets.iter() {
                // Find the parent widget and get the position
                // If no parent element is found assume Pos::ZERO
                let mut parent = tree.path(*widget_id).pop();
                let pos = loop {
                    match parent {
                        None => break Pos::ZERO,
                        Some(p) => match tree.get_ref_by_path(p) {
                            Some(WidgetKind::Element(el)) => break el.container.pos,
                            _ => parent = p.pop(),
                        },
                    }
                };

                tree.with_nodes_and_values(*widget_id, |widget, children, values| {
                    let WidgetKind::Element(el) = widget else { unreachable!("this is always a floating widget") };
                    let mut layout_ctx = LayoutCtx::new(string_storage.new_session(), &attribute_storage, &viewport);

                    layout_widget(el, children, values, self.constraints, &mut layout_ctx, true);

                    // Position
                    position_widget(pos, el, children, values, &attribute_storage, true);

                    // Paint
                    let mut string_session = string_storage.new_session();
                    backend.paint(el, children, values, &mut string_session, &attribute_storage, true);
                });
            }

            backend.render();
            backend.clear();

            // Cleanup removed attributes
            for key in tree.drain_removed() {
                attribute_storage.remove(key);
                floating_widgets.remove(key);
            }

            let sleep = sleep_micros.saturating_sub(fps_now.elapsed().as_micros()) as u64;
            if sleep > 0 {
                std::thread::sleep(Duration::from_micros(sleep));
            }

            fps_now = Instant::now();
        }
    }

    pub fn register_default_widget<W: 'static + Widget + Default>(&mut self, ident: &str) {
        self.factory.register_default::<W>(ident);
    }
}

fn global_event<T: Backend>(
    backend: &mut T,
    tab_indices: &mut TabIndex,
    event: Event,
    tree: &mut WidgetTree<'_>,
    states: &mut States,
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
            if let Some(WidgetKind::Component(component)) = tree.get_mut_by_id(entry.widget_id) {
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_blur(state);
            }
        }

        if let Some(entry) = tab_indices.current() {
            if let Some(WidgetKind::Component(component)) = tree.get_mut_by_id(entry.widget_id) {
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_focus(state);
            }
        }
    }
    event
}

fn handle_messages(
    message_receiver: &Receiver<ViewMessage>,
    tab_indices: &TabIndex,
    fps_now: Instant,
    sleep_micros: u128,
    tree: &mut WidgetTree<'_>,
    states: &mut States,
) -> Duration {
    while let Ok(msg) = message_receiver.try_recv() {
        if let Some(entry) = tab_indices.dumb_fetch(msg.recipient) {
            if let Some(WidgetKind::Component(component)) = tree.get_mut_by_id(entry.widget_id) {
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_message(msg.payload, state);
            }
        }

        // Make sure event handling isn't holding up the rest of the event loop.
        if fps_now.elapsed().as_micros() > sleep_micros / 2 {
            break;
        }
    }

    fps_now.elapsed()
}

fn apply_futures<'bp>(
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
    changes: &mut Changes,
    factory: &Factory,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    components: &mut ComponentRegistry,
    attribute_storage: &mut AttributeStorage<'bp>,
    floating_widgets: &mut FloatingWidgets,
) {
    drain_changes(changes);

    let mut scope = Scope::new();
    changes.drain().rev().for_each(|(sub, change)| {
        sub.iter().for_each(|sub| {
            scope.clear();
            let Some(path) = tree.try_path(sub).cloned() else { return };

            update_tree(
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
