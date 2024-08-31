use std::fs::read_to_string;

use anathema::backend::tui::TuiBackend;
use anathema::runtime::Runtime;
use anathema::templates::Document;

fn main() {
    let template = read_to_string("examples/templates/basic/basic.aml").unwrap();

    let doc = Document::new(template);

    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(doc, backend).finish().unwrap();
    runtime.run();
}
