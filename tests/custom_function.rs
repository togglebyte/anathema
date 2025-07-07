use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;
use anathema_testutils::{BasicComp, BasicState, character};
use anathema_value_resolver::ValueKind;

fn keypress(_: KeyEvent, _: &mut BasicState, _: Children<'_, '_>, _: Context<'_, '_, BasicState>) {}

#[test]
fn run_custom_function() {
    let tpl = "text state.string.custom()";
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

    builder.register_function("custom", custom).unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    assert!(backend.at(0, 0).is_char('x'));

    match res {
        Ok(_) | Err(Error::Stop) => (),
        Err(err) => panic!("{err}"),
    }
}

fn custom<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut buffer = String::new();
    buffer.push('x');

    ValueKind::Str(buffer.into())
}
