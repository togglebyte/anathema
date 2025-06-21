use anathema::component::*;
use anathema::prelude::*;
use anathema_backend::testing::TestBackend;
use anathema_testutils::character;

#[derive(Debug, State)]
struct S {
    value: Value<bool>,
}

impl Default for S {
    fn default() -> Self {
        Self { value: true.into() }
    }
}

#[derive(Debug, Default)]
struct C;

impl Component for C {
    type Message = ();
    type State = S;

    fn on_key(
        &mut self,
        _: KeyEvent,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        context.components.by_name("comp").send(());
    }

    fn on_message(
        &mut self,
        _: Self::Message,
        state: &mut Self::State,
        _: Children<'_, '_>,
        _: Context<'_, '_, Self::State>,
    ) {
        state.value.set(false);
    }
}

#[test]
fn eval_if() {
    let tpl = "
    if state.value
        @inner
    else
        text 'bork'
    ";

    let mut backend = TestBackend::new((10, 3));

    backend
        .events()
        .next()
        .press(character('x'))
        .next_frames(10)
        .press(character('x'))
        .stop();

    let doc = Document::new("@comp");

    let mut builder = Runtime::builder(doc, &backend);
    builder.default::<C>("comp", tpl.to_template()).unwrap();
    builder.default::<C>("inner", "text '1'".to_template()).unwrap();

    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();
}
