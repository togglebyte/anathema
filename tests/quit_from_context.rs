use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;
use anathema_testutils::{BasicComp, BasicState, character};

fn keypress(_: KeyEvent, _: &mut BasicState, _: Children<'_, '_>, mut ctx: Context<'_, '_, BasicState>) {
    ctx.stop_runtime();
}

#[test]
fn quit_from_context() {
    let tpl = "text state.number";
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));

    backend.events().next().press(character(' ')).next();

    let mut builder = Runtime::builder(doc, &backend);
    builder
        .component(
            "index",
            tpl.to_template(),
            BasicComp::<_, BasicState>::new(keypress),
            BasicState::default(),
        )
        .unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    match res {
        Ok(_) | Err(Error::Stop) => (),
        Err(err) => panic!("{err}"),
    }
}
