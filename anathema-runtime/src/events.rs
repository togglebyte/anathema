use anathema_widgets::components::deferred::DeferredComponents;
use anathema_widgets::components::events::{Event, KeyCode, KeyEvent};
use anathema_widgets::tabindex::TabIndex;
use anathema_widgets::Components;

pub trait GlobalEventHandler {
    fn handle(
        &self,
        event: Event,
        tabindex: &mut TabIndex<'_, '_>,
        components: &mut DeferredComponents,
    ) -> Option<Event>;
}

impl GlobalEventHandler for () {
    fn handle(&self, event: Event, tabindex: &mut TabIndex<'_, '_>, _: &mut DeferredComponents) -> Option<Event> {
        if let Event::Key(KeyEvent { code: KeyCode::Tab, ctrl: false, .. }) = event {
            tabindex.next();
            return None;
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::BackTab, ..
        }) = event
        {
            tabindex.prev();
            return None;
        }

        if event.is_ctrl_c() {
            return Some(Event::Stop);
        }
        Some(event)
    }
}

impl<T> GlobalEventHandler for T
where
    T: Fn(Event, &mut TabIndex<'_, '_>, &mut DeferredComponents) -> Option<Event>,
{
    fn handle(
        &self,
        event: Event,
        tabindex: &mut TabIndex<'_, '_>,
        components: &mut DeferredComponents,
    ) -> Option<Event> {
        self(event, tabindex, components)
    }
}
