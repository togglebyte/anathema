use anathema_backend::tui::TuiBackend;
use anathema_backend::Backend;
use anathema_default_widgets::Text;
use anathema_state::state::State;
use anathema_templates::{Document, ToSourceKind};
use anathema_testing::TestRunner;
use anathema_widgets::components::Component;

#[derive(Debug)]
struct S {
    name: anathema_state::Value<String>,
}

impl Default for S {
    fn default() -> Self {
        Self {
            name: "Bob".to_string().into(),
        }
    }
}

impl State for S {
    fn to_common(&self) -> Option<anathema_state::CommonVal<'_>> {
        None
    }

    fn state_get(
        &self,
        path: anathema_state::Path<'_>,
        sub: anathema_state::Subscriber,
    ) -> Option<anathema_state::ValueRef> {
        match path {
            anathema_state::Path::Key("name") => Some(self.name.value_ref(sub)),
            _ => None,
        }
    }

    fn state_lookup(&self, path: anathema_state::Path<'_>) -> Option<anathema_state::PendingValue> {
        match path {
            anathema_state::Path::Key("name") => Some(self.name.to_pending()),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct Index;

impl Component for Index {
    type Message = ();
    type State = S;
}

fn main() {
    let mut backend = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish()
        .unwrap();
    backend.finalize();

    let document = Document::new("@index");

    let mut builder = anathema_runtime::builder::Builder::new(document);
    builder.from_default::<Index>("index", "text 'hello ' state.name ' and then some    '".to_template());

    builder.finish(backend.size(), |mut runtime| {
        let mut frame = runtime.next_frame()?;
        let t1 = frame.tick(&mut backend);
        let t2 = frame.present(&mut backend);
        frame.cleanup();

        std::thread::sleep_ms(1000);
        Ok(())
    });
}
