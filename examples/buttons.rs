use anathema_backend::tui::TuiBackend;
use anathema_runtime::Runtime;
use anathema_state::{CommonVal, State, Value};
use anathema_templates::Document;
use anathema_widgets::components::events::{KeyCode, KeyEvent, KeyState};
use anathema_widgets::components::{Component, Context};
use anathema_widgets::Elements;

struct App;

#[derive(State)]
struct AppState {
    number: Value<i32>,
}

impl Component for App {
    type Message = ();
    type State = AppState;

    fn receive(
        &mut self,
        ident: &str,
        _value: CommonVal<'_>,
        state: &mut Self::State,
        _elements: Elements<'_, '_>,
        _context: Context<'_, Self::State>,
    ) {
        if ident == "increment" {
            *state.number.to_mut() += 1;
        } else if ident == "decrement" {
            *state.number.to_mut() -= 1;
        }
    }

    fn accept_focus(&self) -> bool {
        false
    }
}

struct Button;

#[derive(State)]
struct ButtonState {
    caption: Value<String>,
    in_focus: Value<bool>,
}

impl Component for Button {
    type Message = ();
    type State = ButtonState;

    fn on_blur(&mut self, state: &mut Self::State, _elements: Elements<'_, '_>, _context: Context<'_, Self::State>) {
        state.in_focus.set(false);
    }

    fn on_focus(&mut self, state: &mut Self::State, _elements: Elements<'_, '_>, _context: Context<'_, Self::State>) {
        state.in_focus.set(true);
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        _state: &mut Self::State,
        _elements: Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if matches!(key.state, KeyState::Press) {
            if let KeyCode::Enter = key.code {
                context.publish("click", |state| &state.caption)
            }
        }
    }
}

fn main() {
    let doc = Document::new("@main");

    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(doc, backend);

    runtime
        .register_component(
            "main",
            "examples/templates/buttons/buttons.aml",
            App,
            AppState { number: 0.into() },
        )
        .unwrap();

    runtime
        .register_prototype(
            "button",
            "examples/templates/buttons/button.aml",
            move || Button,
            || ButtonState {
                caption: String::from("lark").into(),
                in_focus: false.into(),
            },
        )
        .unwrap();

    let mut runtime = runtime.finish().unwrap();
    runtime.run();
}
