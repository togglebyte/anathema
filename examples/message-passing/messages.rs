use anathema::component::Component;
use anathema::state::{List, State, Value};

#[derive(Default, State)]
pub struct MessagesState {
    messages: Value<List<String>>,
}

#[derive(Default)]
pub struct Messages;

impl Component for Messages {
    type Message = String;
    type State = MessagesState;

    // Anathema's runtime handles sending the messages to the right recipient
    // so we only need to handle what to do when we receive a message.
    //
    // Imagine having some more involved logic here, like cleaning up old messages
    // if this was a livestream chat, for example
    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        state.messages.push_back(message);
    }
}
