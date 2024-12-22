use anathema_backend::tui::TuiBackend;
use anathema_templates::Document;
use anathema_testing::TestRunner;

fn main() {
    let document = Document::new("text 'hello'");
    let mut builder = anathema_runtime::builder::Builder::new(document);

    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();

    builder.finish(backend, |mut runtime| {
        let mut frame = runtime.next_frame()?;
        frame.tick();
        frame.present();
        Ok(())
    });
}
