use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use testutils::{BasicComp, BasicState, character};

fn keypress(_: KeyEvent, state: &mut BasicState, _: Children<'_, '_>, _: Context<'_, '_, BasicState>) {
    state.number.set(9);
}

#[test]
fn state_change() {
    let tpl = "text state.number";
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));

    backend.events().next().press(character(' ')).next().stop();

    let mut builder = Runtime::builder(doc, &backend);
    builder
        .component(
            "index",
            tpl.to_template(),
            BasicComp::<_, BasicState>::new(keypress),
            BasicState::default(),
        )
        .unwrap();

    let res = builder.finish(|runtime| runtime.run(&mut backend));

    assert!(backend.at(0, 0).is_char('9'));

    if let Err(e) = res {
        panic!("{e}");
    }
}
