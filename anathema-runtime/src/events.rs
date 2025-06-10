use anathema_widgets::components::deferred::DeferredComponents;
use anathema_widgets::components::events::{ComponentEvent, KeyCode, KeyEvent};
use anathema_widgets::tabindex::TabIndex;

pub trait GlobalEventHandler {
    fn handle(
        &self,
        event: ComponentEvent,
        tabindex: &mut TabIndex<'_, '_>,
        components: &mut DeferredComponents,
    ) -> Option<ComponentEvent>;
}

impl GlobalEventHandler for () {
    fn handle(
        &self,
        event: ComponentEvent,
        tabindex: &mut TabIndex<'_, '_>,
        _: &mut DeferredComponents,
    ) -> Option<ComponentEvent> {
        if let ComponentEvent::Key(KeyEvent {
            code: KeyCode::Tab,
            ctrl: false,
            ..
        }) = event
        {
            tabindex.next();
            return None;
        }

        if let ComponentEvent::Key(KeyEvent {
            code: KeyCode::BackTab, ..
        }) = event
        {
            tabindex.prev();
            return None;
        }

        if event.is_ctrl_c() {
            return Some(ComponentEvent::Stop);
        }
        Some(event)
    }
}

impl<T> GlobalEventHandler for T
where
    T: Fn(ComponentEvent, &mut TabIndex<'_, '_>, &mut DeferredComponents) -> Option<ComponentEvent>,
{
    fn handle(
        &self,
        event: ComponentEvent,
        tabindex: &mut TabIndex<'_, '_>,
        components: &mut DeferredComponents,
    ) -> Option<ComponentEvent> {
        self(event, tabindex, components)
    }
}
