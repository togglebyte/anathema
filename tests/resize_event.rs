use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;

#[derive(Debug, State, Default)]
struct S {
    width: Value<u16>,
    height: Value<u16>,
}

#[derive(Debug, Default)]
struct C;

impl Component for C {
    type Message = ();
    type State = S;

    #[allow(unused_variables, unused_mut)]
    fn on_resize(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        let size = children.elements().first(|el, _| el.size()).unwrap();
        state.width.set(size.width);
        state.height.set(size.height);
    }
}

#[test]
fn resize_event() {
    let mut backend = TestBackend::new((10, 3));
    let tpl = "
    expand
        text state.width * state.height
    ";

    backend.events().next().resize((2, 2)).next().stop();

    let doc = Document::new("@comp");

    let mut builder = Runtime::builder(doc, &backend);
    builder.default::<C>("comp", tpl.to_template()).unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    // Resize event should be triggered after cycle so that the new size is available
    assert_eq!(backend.line(0), "4");

    match res {
        Ok(_) | Err(Error::Stop) => (),
        Err(err) => panic!("{err}"),
    }
}
