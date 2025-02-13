use anathema_widgets::components::deferred::DeferredComponents;
use anathema_widgets::components::events::Event;

pub trait GlobalEventHandler {
    fn handle(&self, event: Event, components: &mut DeferredComponents) -> Option<Event>;
}

impl GlobalEventHandler for () {
    fn handle(&self, event: Event, components: &mut DeferredComponents) -> Option<Event> {
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
