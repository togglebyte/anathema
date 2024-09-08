use std::time::{Duration, Instant};

use anathema_backend::Backend;
use anathema_geometry::Size;
use anathema_state::{AnyState, States};
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent, KeyState};
use anathema_widgets::components::{AssociatedEvents, FocusQueue, UntypedContext};
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{AttributeStorage, Components, DirtyWidgets, WidgetKind, WidgetTree};

use crate::error::{Error, Result};
use crate::tree::Tree;

pub(super) struct EventHandler;

impl EventHandler {
    pub(super) fn handle<'bp>(
        &mut self,
        poll_duration: Duration,
        fps_now: Instant,
        sleep_micros: u128,
        backend: &mut impl Backend,
        viewport: &mut Viewport,
        tree: &mut WidgetTree<'bp>,
        constraints: &mut Constraints,
        event_ctx: &mut EventCtx<'_, '_, 'bp>,
    ) -> Result<()> {
        while let Some(event) = backend.next_event(poll_duration) {
            let Some(event) = global_event(event_ctx, backend, tree, event) else {
                return Ok(());
            };

            // Ignore mouse events, as they are handled by global event
            if !event.is_mouse_event() {
                if let Some((widget_id, state_id)) = event_ctx.components.get(event_ctx.components.tab_index) {
                    tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_event(ctx, event));
                }
            }

            match event {
                Event::Resize(width, height) => {
                    let size = Size::from((width, height));
                    backend.resize(size);
                    viewport.resize(size);
                    constraints.set_max_width(size.width);
                    constraints.set_max_height(size.height);

                    // Remember to update the viewport on the context
                    event_ctx.context.viewport = *viewport;

                    // Notify all components of the resize
                    let len = event_ctx.components.len();
                    for i in 0..len {
                        let (widget_id, state_id) = event_ctx
                            .components
                            .get(i)
                            .expect("components can not change during this call");

                        tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_resize(ctx));
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
            while let Some(mut event) = event_ctx.assoc_events.next() {
                event_ctx.states.with_mut(event.state, |state, states| {
                    let common_val = (event.f)(state);
                    let Some(common_val) = common_val.to_common() else { return };
                    let Some(entry) = event_ctx.components.get_by_component_id(event.parent.into()) else {
                        return;
                    };

                    let (widget_id, state_id) = (entry.widget_id, entry.state_id);

                    let strings = event_ctx.context.strings;

                    let mut event_ctx = EventCtx {
                        states,
                        components: event_ctx.components,
                        attribute_storage: event_ctx.attribute_storage,
                        assoc_events: event_ctx.assoc_events,
                        focus_queue: event_ctx.focus_queue,
                        context: event_ctx.context,
                        dirty_widgets: event_ctx.dirty_widgets,
                    };

                    tree.with_component(widget_id, state_id, &mut event_ctx, |comp, ctx| {
                        let event_ident = strings.get_ref_unchecked(event.external);
                        comp.any_receive(ctx, event_ident, common_val)
                    });
                })
            }
        }

        // -----------------------------------------------------------------------------
        //   - Drain focus queue -
        // -----------------------------------------------------------------------------
        while let Some((key, value)) = event_ctx.focus_queue.pop() {
            let len = event_ctx.components.len();
            for i in 0..len {
                let (widget_id, state_id) = event_ctx
                    .components
                    .get(i)
                    .expect("components can not change during this call");

                let found = tree.with_value_mut(widget_id, |_, widget, _| {
                    let WidgetKind::Component(component) = widget else { unreachable!() };

                    let attribs = event_ctx.attribute_storage.get(widget_id);
                    let Some(val) = attribs.get_val(&key) else { return false };
                    let Some(either) = val.load_common_val() else { return false };
                    let Some(cv) = either.to_common() else { return false };
                    if value != cv {
                        return false;
                    }

                    if !component.dyn_component.accept_focus_any() {
                        return false;
                    }

                    true
                });

                // -----------------------------------------------------------------------------
                //   - Blur -
                // -----------------------------------------------------------------------------
                if let Some((widget_id, state_id)) = event_ctx.components.get(event_ctx.components.tab_index) {
                    tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_blur(ctx));
                }

                if found {
                    event_ctx.components.tab_index = i;
                    tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_focus(ctx));
                    break;
                }
            }
        }

        Ok(())
    }
}

// TODO: rename this, it has nothing to do with the events,
// but rather calling functions on dyn components
pub(crate) struct EventCtx<'a, 'rt, 'bp> {
    pub dirty_widgets: &'a mut DirtyWidgets,
    pub components: &'a mut Components,
    pub states: &'a mut States,
    pub attribute_storage: &'a mut AttributeStorage<'bp>,
    pub assoc_events: &'a mut AssociatedEvents,
    pub focus_queue: &'a mut FocusQueue<'static>,
    pub context: UntypedContext<'rt>,
}

fn global_event<'bp, T: Backend>(
    event_ctx: &mut EventCtx<'_, '_, 'bp>,
    backend: &mut T,
    tree: &mut WidgetTree<'bp>,
    event: Event,
) -> Option<Event> {
    // -----------------------------------------------------------------------------
    //   - Ctrl-c to quite -
    //   This should be on by default.
    //   Give it a good name
    //
    //   TODO: Do away with this thing once we add a global event handler
    // -----------------------------------------------------------------------------
    if backend.quit_test(event) {
        return Some(Event::Stop);
    }

    // -----------------------------------------------------------------------------
    //   - Handle tabbing between components -
    // -----------------------------------------------------------------------------
    if let Event::Key(KeyEvent {
        code,
        state: KeyState::Press,
        ..
    }) = event
    {
        enum Dir {
            F,
            B,
        }

        let index = event_ctx.components.tab_index;
        let dir = match code {
            KeyCode::Tab => Dir::F,
            KeyCode::BackTab => Dir::B,
            _ => return Some(event),
        };

        loop {
            // -----------------------------------------------------------------------------
            //   - Blur -
            // -----------------------------------------------------------------------------
            if let Some((widget_id, state_id)) = event_ctx.components.get(event_ctx.components.tab_index) {
                tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_blur(ctx));
            }

            // -----------------------------------------------------------------------------
            //   - Change index -
            // -----------------------------------------------------------------------------
            match dir {
                Dir::F => {
                    event_ctx.components.tab_index += 1;
                    if event_ctx.components.tab_index >= event_ctx.components.len() {
                        event_ctx.components.tab_index = 0;
                    }
                }
                Dir::B => match event_ctx.components.tab_index >= 1 {
                    true => event_ctx.components.tab_index -= 1,
                    false => event_ctx.components.tab_index = event_ctx.components.len() - 1,
                },
            }

            if index == event_ctx.components.tab_index {
                break;
            }

            // -----------------------------------------------------------------------------
            //   - Focus -
            // -----------------------------------------------------------------------------
            if let Some((widget_id, state_id)) = event_ctx.components.current() {
                tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_focus(ctx));

                let cont = tree
                    .with_component(widget_id, state_id, event_ctx, |comp, ctx| {
                        if !comp.accept_focus_any() {
                            return true;
                        }
                        comp.any_focus(ctx);
                        false
                    })
                    .unwrap_or(true);

                if !cont {
                    break;
                }
            }
        }

        return None;
    }

    // Mouse events are global
    if let Event::Mouse(_) = event {
        for i in 0..event_ctx.components.len() {
            let (widget_id, state_id) = event_ctx
                .components
                .get(i)
                .expect("components can not change during this call");

            tree.with_component(widget_id, state_id, event_ctx, |comp, ctx| comp.any_event(ctx, event));
        }
    }

    Some(event)
}
