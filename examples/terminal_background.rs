use std::fs::read_to_string;

use anathema::backend::Backend;
use anathema::backend::tui::TuiBackend;
use anathema::runtime::Runtime;
use anathema::state::Color;
use anathema::templates::{Document, ToSourceKind};

fn main() {
    let template = read_to_string("examples/templates/basic/basic.aml").unwrap();

    let doc = Document::new("@index");

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    // This is not set on the builder as it can be called at any time
    // to change the background color of the terminal.
    backend.set_background_color(Color::Rgb(135, 105, 20));

    let mut builder = Runtime::builder(doc, &backend);
    builder.template("index", template.to_template()).unwrap();
    let _ = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));
}
