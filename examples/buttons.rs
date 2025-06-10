use anathema::component::*;
use anathema::prelude::*;

struct App;

#[derive(State)]
struct AppState {
    number: Value<i32>,
}

impl Component for App {
    type Message = ();
    type State = AppState;

    const TICKS: bool = false;

    fn on_event(
        &mut self,
        ident: &str,
        _: &dyn State,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
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
    active: Value<u8>,
}

impl Component for Button {
    type Message = ();
    type State = ButtonState;

    const TICKS: bool = false;

    fn on_blur(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        state.active.set(0);
    }

    fn on_focus(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        state.active.set(1);
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        if !matches!(key.state, KeyState::Press) {
            return;
        }

        if let KeyCode::Enter = key.code {
            context.publish("click")
        }
    }
}

fn main() {
    let doc = Document::new("@main");

    let mut backend = TuiBackend::builder()
        // .enable_alt_screen()
        .enable_raw_mode()
        .clear()
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
            || ButtonState { active: 0.into() },
        )
        .unwrap();

    builder.finish(|runtime| runtime.run(&mut backend)).unwrap();
}
