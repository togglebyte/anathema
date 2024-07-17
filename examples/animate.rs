use std::f64::consts::PI;
use std::fs::read_to_string;
use std::time::Duration;

use anathema::component::*;
use anathema::prelude::*;

#[derive(State)]
struct Num {
    x: Value<f64>,
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

    fn tick(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>, dt: Duration) {
        let x = dt.as_millis() as f64;

        self.val += x / 1000.0 * *state.speed.to_ref();
        let x = ease_in_out(self.val) * (context.viewport.size().width - 8) as f64;
        state.x.set(x);
    }

    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>) {
        match key.code {
            KeyCode::Char('k') => *state.speed.to_mut() += 0.1,
            KeyCode::Char('j') => *state.speed.to_mut() -= 0.1,
            _ => {}
        }
    }
}

fn main() {
    let template = read_to_string("examples/templates/animate.aml").unwrap();

    let mut doc = Document::new("@main");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::new(doc, backend);
    runtime.register_component(
        "main",
        template,
        C { val: 0.0 },
        Num {
            x: 0.0.into(),
            speed: 0.1.into(),
        },
    );
    let mut runtime = runtime.finish().unwrap();
    runtime.fps = 40;
    runtime.run();
}
