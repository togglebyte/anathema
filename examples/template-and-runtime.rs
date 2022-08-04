use std::fs::read_to_string;

use anathema::runtime::{Event, Runtime};
use anathema::templates::DataCtx;

static TEMPLATE: &'static str = r#"
border:
    text: "I would like a hot cup of tea"
"#;

fn main() {
    let mut runtime = Runtime::<()>::new();
    runtime.start(TEMPLATE, DataCtx::empty(), |event, ctx, root_widget, tx| {
        if event.ctrl_c() {
            let _ = tx.send(Event::Quit);
        }
    });
}
