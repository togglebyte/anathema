use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_testutils::{BasicComp, character};

#[derive(Debug, State)]
struct Inner {
    list: Value<List<u32>>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            list: List::from_iter([1]).into(),
        }
    }
}

#[derive(Debug, State, Default)]
struct Outer {
    inner: Value<Inner>,
}

type Comp<F> = BasicComp<F, Outer>;

fn keypress(_: KeyEvent, state: &mut Outer, _: Children<'_, '_>, _: Context<'_, '_, Outer>) {
    state.inner.set(Inner {
        list: List::from_iter([2]).into(),
    });
}

#[test]
fn nested_collections() {
    let tpl = "
    for i in state.inner.list
        text i
    ";
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));

    backend.events().next().press(character(' ')).next().stop();

    let mut builder = Runtime::builder(doc, &backend);
    builder
        .component("index", tpl.to_template(), Comp::<_>::new(keypress), Outer::default())
        .unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| runtime.run(backend));

    assert!(backend.at(0, 0).is_char('2'));

    if let Err(e) = res {
        panic!("{e}");
    }
}
