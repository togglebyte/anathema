mod index;
mod messages;

use anathema::backend::tui::TuiBackend;
use anathema::runtime::Runtime;
use anathema::templates::Document;
use index::Index;
use messages::Messages;

fn main() {
    let doc = Document::new("@index");
    let backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .enable_mouse()
        .hide_cursor()
        .finish()
        .unwrap();

    let mut runtime = Runtime::builder(doc, backend);

    let recipient = runtime
        .register_default::<Messages>("messages", "examples/message_passing/templates/messages.aml")
        .expect("failed to register messages component");

    runtime
        .register_component(
            "index",
            "examples/message_passing/templates/index.aml",
            Index::new(recipient),
            (),
        )
        .expect("failed to register index component");

    runtime.finish().unwrap().run();
}
