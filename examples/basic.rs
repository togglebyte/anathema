use std::fs::read_to_string;

use anathema::backend::Backend;
use anathema::backend::tui::TuiBackend;
use anathema::runtime::Runtime;
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

    let mut builder = Runtime::builder(doc, &backend);
    builder.template("index", template.to_template()).unwrap();
    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}
