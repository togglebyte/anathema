use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;
use testutils::{char_press, Thing};

#[derive(Debug, Default)]
struct Comp;

impl Component for Comp {
    type Message = ();
    type State = Thing;

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        state.value.set(9);
    }
}

#[test]
fn lol() {
    let tpl = "text state.value";
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));

    backend.add_event(None);
    backend.add_event(Some(ComponentEvent::Key(char_press(' '))));
    backend.add_event(Some(ComponentEvent::Stop));

    let mut builder = Runtime::builder(doc, &backend);
    builder.from_default::<Comp>("index", tpl.to_template()).unwrap();

    let res = builder.finish(|runtime| runtime.run(&mut backend));

    assert!(backend.at(0, 0).is_char('9'));

    if let Err(e) = res {
        panic!("{e}");
    }
}
