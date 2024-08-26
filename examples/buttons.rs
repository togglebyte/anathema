use std::ops::Add;
use anathema_backend::tui::TuiBackend;
use anathema_runtime::Runtime;
use anathema_state::{AnyState, CommonVal, State, Value};
use anathema_templates::Document;
use anathema_widgets::components::{Component, ComponentId, Context};
use anathema_widgets::components::events::{KeyCode, KeyEvent, KeyState};
use anathema_widgets::Elements;
use strum_macros::{Display, EnumString, IntoStaticStr};

struct App;

#[derive(State)]
struct AppState {
    number: Value<i32>,
}
impl Component for App {
    type Message = AppMessage;
    type State = AppState;

    fn message(&mut self, message: Self::Message, state: &mut Self::State, elements: Elements<'_, '_>, context: Context<'_>) {
        match message {
            AppMessage::Increment => {
                let number = state.number.to_ref().add(1);
                state.number.set(number);
            }
            AppMessage::Decrement => {}
        }
    }
}

#[derive(EnumString, IntoStaticStr, Copy, Clone)]
enum AppMessage {
    Increment, Decrement,
}

impl State for AppMessage {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        Some(CommonVal::Str(<&str>::from(self)))
    }
}

struct Button(ComponentId<AppMessage>);

#[derive(State)]
struct ButtonState {
    caption: Value<String>,
    in_focus: Value<bool>,
    message: Value<AppMessage>,
}

impl Component for Button {
    type Message = ();
    type State = ButtonState;

    fn on_focus(&mut self, state: &mut Self::State, elements: Elements<'_, '_>, context: Context<'_>) {
        state.in_focus.set(true);
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State, elements: Elements<'_, '_>, context: Context<'_>) {
        if matches!(key.state, KeyState::Press) {
            match key.code {
                KeyCode::Enter => {
                    let emitter = context.emitter;
                    emitter.emit(self.0, state.message.copy_value()).unwrap()
                },
                _ => ()
            }
        }
    }

    fn on_blur(&mut self, state: &mut Self::State, elements: Elements<'_, '_>, context: Context<'_>) {
        state.in_focus.set(false);
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
    let app_id = runtime
        .register_component(
            "main",
            "examples/templates/buttons.aml",
            App,
            AppState { number: 0.into() },
        )
        .unwrap();
    runtime
        .register_prototype(
            "button",
            "examples/templates/button.aml",
            move || Button(app_id),
            || ButtonState {
                caption: String::from("lark").into(),
                in_focus: false.into(),
                message: AppMessage::Increment.into(),
            },
        )
        .unwrap();

    let mut runtime = runtime.finish().unwrap();
    runtime.run();
}
