use std::f64::consts::PI;
use std::time::Duration;

use anathema::component::*;
use anathema::prelude::*;

#[derive(State)]
struct Num {
    x: Value<i32>,
    speed: Value<f64>,
}

struct C {
    val: f64,
}

fn ease_in_out(val: f64) -> f64 {
    let x = PI * val;
    let x = x.cos() - 1.0;
    -(x / 2.0)
}

impl Component for C {
    type Message = ();
    type State = Num;

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        _: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        dt: Duration,
    ) {
        let x = dt.as_millis() as f64;

        self.val += x / 1000.0 * *state.speed.to_ref();
        let x = ease_in_out(self.val) * (context.viewport.size().width - 8) as f64;
        state.x.set(x as i32);
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        if matches!(key.state, KeyState::Press) {
            match key.code {
                KeyCode::Char('k') => *state.speed.to_mut() += 0.1,
                KeyCode::Char('j') => *state.speed.to_mut() -= 0.1,
                _ => {}
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
            "examples/templates/animate/animate.aml",
            C { val: 0.0 },
            Num {
                x: 0.into(),
                speed: 0.1.into(),
            },
        )
        .unwrap();

    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}
