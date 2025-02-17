use anathema::prelude::*;
use anathema::component::*;

struct App;

#[derive(State)]
struct AppState {
    number: Value<i32>,
}

impl Component for App {
    type Message = ();
    type State = AppState;

    const TICKS: bool = false;

    fn receive(
        &mut self,
        ident: &str,
        value: &dyn AnyState,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, Self::State>,
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

    const TICKS: bool = false;

    fn on_blur(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state.in_focus.set(false);
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        state.in_focus.set(true);
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if matches!(key.state, KeyState::Press) {
            if let KeyCode::Enter = key.code {
                // context.publish("click", |state| &state.caption)
                context.publish("click")
            }
        }
    }
}

fn main() {
    let doc = Document::new("@main");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    let mut builder = Runtime::builder(doc, &backend);

    builder
        .component(
            "main",
            "examples/templates/buttons/buttons.aml",
            App,
            AppState { number: 0.into() },
        )
        .unwrap();

    builder
        .prototype(
            "button",
            "examples/templates/buttons/button.aml",
            move || Button,
            || ButtonState {
                caption: String::from("lark").into(),
                in_focus: false.into(),
            },
        )
        .unwrap();

    builder
        .finish(|mut runtime| runtime.run(&mut backend))
        .unwrap();
}
