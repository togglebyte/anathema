use anathema::backend::tui::TuiBackend;
use anathema::component::{Children, Component, Context, MouseEvent};
use anathema::runtime::Runtime;
use anathema::state::{List, State, Value};
use anathema::templates::Document;
use anathema_backend::Backend;

pub struct Index;

impl Component for Index {
    type Message = ();
    type State = ();

    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        _: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if mouse.left_down() {
            children
                .elements()
                .at_position(mouse.pos())
                .by_attribute("id", "button")
                .first(|_, attr| {
                    let Some(value) = attr.get_as::<&str>("id").map(|s| s.to_string()) else { return };
                    context.components.by_name("messages").send(value);
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

    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
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
    let mut backend = TuiBackend::builder()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .clear()
        .finish()
        .unwrap();
    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);

    builder
        .default::<Messages>("messages", "examples/templates/message-passing/messages.aml")
        .expect("failed to register messages component");

    builder
        .component(
            "index",
            "examples/templates/message-passing/message_passing.aml",
            Index,
            (),
        )
        .expect("failed to register index component");

    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}
