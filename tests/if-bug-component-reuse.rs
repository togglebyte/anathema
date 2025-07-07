use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;
use anathema_testutils::{BasicComp, BasicState, character};

// Bug triggered when a component is used in two separate branches:
//
// ```
// if x
//     @comp [val: 1]
// else
//     @comp [val: 2]
// ```

static TEMPLATE: &str = "
    if state.number == 0
        @comp [val: !state.boolean]
    else
        @comp [val: state.boolean]
";

fn keypress(_: KeyEvent, state: &mut BasicState, _: Children<'_, '_>, _: Context<'_, '_, BasicState>) {
    *state.number.to_mut() += 1;
}

#[test]
fn bug_component_reuse_bug() {
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));
    backend
        .events()
        .next()
        .press(character('x'))
        .next()
        .press(character('x'))
        .next()
        .stop();

    let mut builder = Runtime::builder(doc, &backend);
    builder
        .component(
            "index",
            TEMPLATE.to_template(),
            BasicComp::<_>::new(keypress),
            BasicState::default(),
        )
        .unwrap();
    builder
        .component("comp", "text attributes.val".to_template(), (), ())
        .unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    assert_eq!(backend.line(0), "false");

    match res {
        Ok(_) | Err(Error::Stop) => (),
        Err(err) => panic!("{err}"),
    }
}
