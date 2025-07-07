use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;

struct Comp;

impl Component for Comp {
    type Message = ();
    type State = bool;

    fn on_mount(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        *state = true;
    }

    fn on_unmount(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        if *state {
            panic!("test passed");
        }
    }
}

#[test]
#[should_panic(expected = "test passed")]
fn state_change() {
    let tpl = "text state";
    let doc = Document::new("@index");

    let mut backend = TestBackend::new((10, 3));

    backend.events().next().stop();

    let mut builder = Runtime::builder(doc, &backend);
    builder.component("index", tpl.to_template(), Comp, true).unwrap();

    let res = builder.finish(&mut backend, |runtime, backend| {
        runtime.with_frame(backend, |backend, mut frame| {
            frame.tick(backend)?;
            frame.force_unmount_return();
            Err(Error::Stop)
        })
    });

    assert_eq!(backend.line(0), "true");

    match res {
        Ok(_) | Err(Error::Stop) => (),
        Err(err) => panic!("{err}"),
    }
}
