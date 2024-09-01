use anathema::backend::tui::TuiBackend;
use anathema::component::{Component, ComponentId, MouseEvent};
use anathema::runtime::Runtime;
use anathema::state::{List, State, Value};
use anathema::templates::Document;
use anathema::widgets::components::Context;
use anathema::widgets::Elements;

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
        mouse: MouseEvent,
        _state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        if mouse.lsb_down() {
            elements
                .at_position(mouse.pos())
                .by_attribute("id", "button")
                .first(|_, _| {
                    context
                        .emitter
                        .emit(self.recipient, "hey look, thats a message!".into())
                        .unwrap();
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

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Elements<'_, '_>,
        _: Context<'_, Self::State>,
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
        .register_default::<Messages>("messages", "examples/templates/message-passing/messages.aml")
        .expect("failed to register messages component");

    runtime
        .register_component(
            "index",
            "examples/templates/message-passing/message_passing.aml",
            Index::new(recipient),
            (),
        )
        .expect("failed to register index component");

    runtime.finish().unwrap().run();
}
