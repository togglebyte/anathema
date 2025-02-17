use anathema_widgets::components::deferred::DeferredComponents;
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::Components;

pub trait GlobalEventHandler {
    fn handle(&self, event: Event, components: &mut DeferredComponents) -> Option<Event>;
}

impl GlobalEventHandler for () {
    fn handle(&self, event: Event, _: &mut DeferredComponents) -> Option<Event> {
        if let Event::Key(KeyEvent { code: KeyCode::Tab, .. }) = event {}

        if let Event::Key(KeyEvent {
            code: KeyCode::BackTab, ..
        }) = event
        {}

        if event.is_ctrl_c() {
            return Some(Event::Stop);
        }
        Some(event)
    }
}

impl<T> GlobalEventHandler for T
where
    T: Fn(Event, &mut DeferredComponents) -> Option<Event>,
{
    fn handle(&self, event: Event, components: &mut DeferredComponents) -> Option<Event> {
        self(event, components)
    }
}
