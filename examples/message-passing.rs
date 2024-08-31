use anathema::backend::tui::TuiBackend;
use anathema::component::{Component, ComponentId};
use anathema::runtime::Runtime;
use anathema::state::{List, State, Value};
use anathema::templates::Document;

pub struct Index {
    recipient: ComponentId<String>,
}

impl Index {
    pub fn new(recipient: ComponentId<String>) -> Self {
        Self { recipient }
    }
}

impl Component for Index {
    type Message = ();
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

#[derive(Default, State)]
pub struct MessagesState {
    messages: Value<List<String>>,
    message_count: Value<usize>,
}

#[derive(Default)]
pub struct Messages;

impl Component for Messages {
    type Message = String;
    type State = MessagesState;

    // Anathema's runtime handles sending the messages to the right recipient
    // so we only need to handle what to do when we receive a message.
    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        if state.messages.len() > 20 {
            state.messages.pop_front();
        }
        let message_count = state.message_count.copy_value() + 1;
        state.message_count.set(message_count);
        state.messages.push_back(format!("{message_count} {message}"));
    }
}

fn main() {
    let doc = Document::new("@index");
    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(doc, backend);

    let recipient = runtime
        .register_default::<Messages>("messages", "templates/messages.aml")
        .expect("failed to register messages component");

    runtime
        .register_component("index", "templates/message_passing.aml", Index::new(recipient), ())
        .expect("failed to register index component");

    runtime.finish().unwrap().run();
}
