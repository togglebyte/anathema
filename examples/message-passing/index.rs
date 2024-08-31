use anathema::component::{Component, ComponentId};

pub struct Index {
    recipient: ComponentId<String>,
}

impl Index {
    pub fn new(recipient: ComponentId<String>) -> Self {
        Self { recipient }
    }
}

impl Component for Index {
    type Message = (); // we dont accept messages, we only send them!
    type State = ();

    fn on_mouse(
        &mut self,
        mouse: anathema::component::MouseEvent,
        _state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if mouse.lsb_down() {
            elements
                .at_position(mouse.pos())
                .by_attribute("id", "button")
                .first(|_, _| {
                    // Anathema's context exposes the emitter for us, so we can send
                    // messages that way.
                    _ = context
                        .emitter
                        .emit(self.recipient, "hey look, thats a message!".into());
                });
        }
    }
}
