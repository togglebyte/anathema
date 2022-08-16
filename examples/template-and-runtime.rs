use anathema::runtime::{Event, Runtime};
use anathema::templates::DataCtx;

static TEMPLATE: &str = r#"
border:
    text: "I would like a hot cup of tea"
"#;

fn main() {
    let runtime = Runtime::<()>::new();
    runtime
        .start(TEMPLATE, DataCtx::empty(), |event, _ctx, _root_widget, tx| {
            if event.ctrl_c() {
                let _ = tx.send(Event::Quit);
            }
        })
        .unwrap();
}
