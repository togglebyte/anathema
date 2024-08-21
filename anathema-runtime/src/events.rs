use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_state::{AnyState, States};
use anathema_store::storage::strings::Strings;
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::components::{AssociatedEvents, Context, Emitter};
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{AttributeStorage, Elements, WidgetKind, WidgetTree};

use crate::components::Components;
use crate::error::{Error, Result};

pub(super) struct EventHandler;

impl EventHandler {
    pub(super) fn handle<'bp>(
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
        assoc_events: &mut AssociatedEvents,
        strings: &Strings,
    ) -> Result<()> {
        while let Some(event) = backend.next_event(poll_duration) {
            let Some(event) = global_event(
                backend,
                components,
                event,
                tree,
                states,
                attribute_storage,
                emitter,
                *viewport,
                assoc_events,
                strings,
            ) else {
                return Ok(());
            };

            // Ignore mouse events, as they are handled by global event
            if !event.is_mouse_event() {
                if let Some(entry) = components.current() {
                    tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                        let WidgetKind::Component(component) = widget else { return };
                        let state = states.get_mut(entry.state_id);

                        let parent = component
                            .parent
                            .and_then(|parent| components.dumb_fetch(parent))
                            .map(|parent| parent.widget_id.into());

                        let Some((node, values)) = tree.get_node_by_path(path) else { return };
                        let elements = Elements::new(node.children(), values, attribute_storage);
                        let context = Context {
                            emitter,
                            viewport: *viewport,
                            assoc_events,
                            state_id: entry.state_id,
                            parent,
                            strings,
                            assoc_functions: component.assoc_functions,
                        };
                        component.dyn_component.any_event(event, state, elements, context);
                    });
                }
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
                            let state = states.get_mut(entry.state_id);

                            let parent = component
                                .parent
                                .and_then(|parent| components.dumb_fetch(parent))
                                .map(|parent| parent.widget_id.into());

                            let Some((node, values)) = tree.get_node_by_path(path) else { return };
                            let elements = Elements::new(node.children(), values, attribute_storage);
                            let context = Context {
                                emitter,
                                viewport: *viewport,
                                assoc_events,
                                state_id: entry.state_id,
                                parent,
                                strings,
                                assoc_functions: component.assoc_functions,
                            };
                            component.dyn_component.any_resize(state, elements, context);
                        });
                    }
                }
                Event::Blur => (),
                Event::Focus => (),
                Event::Stop => return Err(Error::Stop),
                _ => {}
            }

            // Make sure event handling isn't holding up the rest of the event loop.
            if fps_now.elapsed().as_micros() > sleep_micros {
                break;
            }

            // -----------------------------------------------------------------------------
            //   - Drain associated events -
            // -----------------------------------------------------------------------------
            while let Some(mut event) = assoc_events.next() {
                states.with_mut(event.state, |state, states| {
                    let common_val = (event.f)(state);
                    let Some(common_val) = common_val.to_common() else { return };
                    let Some(entry) = components.by_widget_id(event.parent.into()) else { return };
                    tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                        let WidgetKind::Component(component) = widget else { return };

                        let event_ident = strings.get_ref_unchecked(event.external);

                        let state = states.get_mut(entry.state_id);

                        let parent = component
                            .parent
                            .and_then(|parent| components.dumb_fetch(parent))
                            .map(|parent| parent.widget_id.into());

                        let Some((node, values)) = tree.get_node_by_path(path) else { return };
                        let elements = Elements::new(node.children(), values, attribute_storage);
                        let context = Context {
                            emitter,
                            viewport: *viewport,
                            assoc_events,
                            state_id: entry.state_id,
                            parent,
                            strings,
                            assoc_functions: component.assoc_functions,
                        };

                        component
                            .dyn_component
                            .any_callback(state, event_ident, common_val, elements, context);
                    });
                })
            }
        }

        Ok(())
    }
}

pub fn global_event<'bp, T: Backend>(
    backend: &mut T,
    components: &mut Components,
    event: Event,
    tree: &mut WidgetTree<'bp>,
    states: &mut States,
    attribute_storage: &mut AttributeStorage<'bp>,
    emitter: &Emitter,
    viewport: Viewport,
    assoc_events: &mut AssociatedEvents,
    strings: &Strings,
) -> Option<Event> {
    // -----------------------------------------------------------------------------
    //   - Ctrl-c to quite -
    //   This should be on by default.
    //   Give it a good name
    // -----------------------------------------------------------------------------
    if backend.quit_test(event) {
        return Some(Event::Stop);
    }

    // -----------------------------------------------------------------------------
    //   - Handle tabbing between components -
    // -----------------------------------------------------------------------------
    if let Event::Key(KeyEvent { code, .. }) = event {
        let prev = match code {
            KeyCode::Tab => components.next(),
            KeyCode::BackTab => components.prev(),
            _ => return Some(event),
        };

        if let Some(entry) = prev {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };

                let parent = component
                    .parent
                    .and_then(|parent| components.dumb_fetch(parent))
                    .map(|parent| parent.widget_id.into());

                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = states.get_mut(entry.state_id);
                let context = Context {
                    emitter,
                    viewport,
                    assoc_events,
                    state_id: entry.state_id,
                    parent,
                    strings,
                    assoc_functions: component.assoc_functions,
                };
                component.dyn_component.any_blur(state, elements, context);
            });
        }

        if let Some(entry) = components.current() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };

                let parent = component
                    .parent
                    .and_then(|parent| components.dumb_fetch(parent))
                    .map(|parent| parent.widget_id.into());

                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = states.get_mut(entry.state_id);
                let context = Context {
                    emitter,
                    viewport,
                    assoc_events,
                    state_id: entry.state_id,
                    parent,
                    strings,
                    assoc_functions: component.assoc_functions,
                };
                component.dyn_component.any_focus(state, elements, context);
            });
        }

        return None;
    }

    // Mouse events are global
    if let Event::Mouse(_) = event {
        for entry in components.iter() {
            tree.with_value_mut(entry.widget_id, |path, widget, tree| {
                let WidgetKind::Component(component) = widget else { return };

                let parent = component
                    .parent
                    .and_then(|parent| components.dumb_fetch(parent))
                    .map(|parent| parent.widget_id.into());

                let Some((node, values)) = tree.get_node_by_path(path) else { return };
                let elements = Elements::new(node.children(), values, attribute_storage);
                let state = states.get_mut(entry.state_id);
                let context = Context {
                    emitter,
                    viewport,
                    assoc_events,
                    state_id: entry.state_id,
                    parent,
                    strings,
                    assoc_functions: component.assoc_functions,
                };
                let _ = component.dyn_component.any_event(event, state, elements, context);
            });
        }
    }

    Some(event)
}
