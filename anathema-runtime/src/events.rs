use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_state::States;
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::components::{Context, Emitter};
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{AttributeStorage, Elements, WidgetKind, WidgetTree};

use crate::components::Components;
use crate::error::{Error, Result};

pub struct EventHandler {}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle<'bp>(
        &mut self,
        poll_duration: Duration,
        fps_now: Instant,
        sleep_micros: u128,
        backend: &mut impl Backend,
        viewport: &mut Viewport,
        emitter: &Emitter,
        tree: &mut WidgetTree<'bp>,
        components: &mut Components,
        states: &mut States,
        attribute_storage: &mut AttributeStorage<'bp>,
        constraints: &mut Constraints,
    ) -> Result<()> {
        while let Some(event) = backend.next_event(poll_duration) {
            let context = Context { emitter, viewport: *viewport };
            let event = global_event(backend, components, event, tree, states, attribute_storage, context);

            // Ignore mouse events, as they are handled by global event
            if !event.is_mouse_event() {
                if let Some(entry) = components.current() {
                    tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                        let WidgetKind::Component(component) = widget else { return };
                        let state = entry.state_id.and_then(|id| states.get_mut(id));
                        let Some((node, values)) = tree.get_node_by_path(path) else { return };
                        let elements = Elements::new(node.children(), values, attribute_storage);
                        let context = Context { emitter, viewport: *viewport };
                        component.component.any_event(event, state, elements, context);
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
                    backend.resize(size);
                    viewport.resize(size);
                    constraints.set_max_width(size.width);
                    constraints.set_max_height(size.height);

                    // Notify all components of the resize
                    for entry in components.iter() {
                        tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                            let WidgetKind::Component(component) = widget else { return };
                            let state = entry.state_id.and_then(|id| states.get_mut(id));
                            let Some((node, values)) = tree.get_node_by_path(path) else { return };
                            let elements = Elements::new(node.children(), values, attribute_storage);
                            let context = Context { emitter, viewport: *viewport };
                            component.component.any_resize(state, elements, context);
                        });
                    }
                }
                Event::Blur => (),
                Event::Focus => (),
                Event::Stop => return Err(Error::Stop),
                _ => {}
            }
        }

        Ok(())
    }
}

pub fn global_event<'bp, T: Backend>(
    backend: &mut T,
    tab_indices: &mut Components,
    event: Event,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    attribute_storage: &mut AttributeStorage<'bp>,
    context: Context<'_>,
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
                component.component.any_blur(state, elements, context);
            });
        }

        if let Some(entry) = tab_indices.current() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };
                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = entry.state_id.and_then(|id| states.get_mut(id));
                component.component.any_focus(state, elements, context);
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
                let _ = component.component.any_event(event, state, elements, context);
            });
        }
    }

    event
}
