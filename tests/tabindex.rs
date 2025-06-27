use anathema::component::*;
use anathema::prelude::*;
use anathema::runtime::Builder;
use anathema_backend::testing::TestBackend;
use anathema_runtime::Error;
use anathema_testutils::{BasicComp, BasicState, character, tab};
use anathema_widgets::WidgetId;
use anathema_widgets::tabindex::Index;

fn keypress(_: KeyEvent, state: &mut BasicState, _: Children<'_, '_>, _: Context<'_, '_, BasicState>) {
    state.number.set(9);
}

fn comp_index(widget_id: u32, index: Option<Index>) {
    let index = index.unwrap();
    assert_eq!(WidgetId::new(widget_id, 0), index.widget_id);
}

fn builder(template: &str, backend: &mut impl Backend) -> Builder<()> {
    let doc = Document::new(template);

    let mut builder = Runtime::builder(doc, backend);
    builder
        .prototype(
            "comp",
            String::new().to_template(),
            || BasicComp::<_, BasicState>::new(keypress),
            BasicState::default,
        )
        .unwrap();

    builder
}

#[test]
fn tabindex_change() {
    let tpl = "
    vstack
        @comp [id: 1]
        @comp [id: 2]
        @comp [id: 3]
    ";

    let mut backend = TestBackend::new((10, 3));

    backend
        .events()
        .next()
        .press(tab())
        .next()
        .press(tab())
        .next()
        .press(tab())
        .next()
        .press(tab())
        .next()
        .stop();

    let builder = builder(tpl, &mut backend);
    builder
        .finish(&mut backend, |runtime, backend| {
            runtime.with_frame(backend, |backend, mut frame| {
                // Initial tick to build the tree
                frame.tick(backend)?;

                assert!(frame.tabindex.is_none());
                frame.tick(backend)?;
                comp_index(1, frame.tabindex.clone());

                frame.tick(backend)?;
                comp_index(2, frame.tabindex.clone());

                frame.tick(backend)?;
                comp_index(3, frame.tabindex.clone());

                frame.tick(backend)?;
                comp_index(1, frame.tabindex.clone());

                Err(Error::Stop)
            })
        })
        .unwrap();
}

#[test]
fn tabindex_single_component() {
    let tpl = "
    vstack
        @comp [id: 1]
    ";

    let mut backend = TestBackend::new((10, 3));

    backend.events().next().press(tab()).next().press(tab()).next().stop();

    let builder = builder(tpl, &mut backend);
    builder
        .finish(&mut backend, |runtime, backend| {
            runtime.with_frame(backend, |backend, mut frame| {
                // Initial tick to build the tree
                frame.tick(backend)?;

                assert!(frame.tabindex.is_none());
                frame.tick(backend)?;
                comp_index(1, frame.tabindex.clone());

                frame.tick(backend)?;
                comp_index(1, frame.tabindex.clone());

                Err(Error::Stop)
            })
        })
        .unwrap();
}

#[test]
fn tabindex_no_component() {
    let tpl = "
    vstack
    ";

    let mut backend = TestBackend::new((10, 3));

    backend.events().press(tab()).next().press(tab()).next().stop();

    let builder = builder(tpl, &mut backend);

    builder
        .finish(&mut backend, |runtime, backend| {
            runtime.with_frame(backend, |backend, mut frame| {
                // Initial tick to build the tree
                assert!(frame.tabindex.is_none());
                frame.tick(backend)?;
                assert!(frame.tabindex.is_none());
                frame.tick(backend)?;
                assert!(frame.tabindex.is_none());
                Err(Error::Stop)
            })
        })
        .unwrap();
}

struct CompA;
impl Component for CompA {
    type Message = ();
    type State = bool;

    fn on_key(
        &mut self,
        _: KeyEvent,
        _: &mut Self::State,
        _: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        context.components.by_name("comp_b").focus();
    }

    fn on_blur(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        *state = false;
    }
}

struct CompB;
impl Component for CompB {
    type Message = ();
    type State = bool;

    fn on_focus(&mut self, state: &mut Self::State, _: Children<'_, '_>, _: Context<'_, '_, Self::State>) {
        *state = true;
    }
}

#[test]
fn tabindex_change_via_deferred_command() {
    let tpl = "
    vstack
        @comp_a
        @comp_b
    ";

    let comp_tpl = "text state";

    let mut backend = TestBackend::new((5, 2));

    backend.events().next().press(character('x')).next().stop();

    let mut builder = builder(tpl, &mut backend);
    builder
        .component("comp_a", comp_tpl.to_template(), CompA, true)
        .unwrap();
    builder
        .component("comp_b", comp_tpl.to_template(), CompB, false)
        .unwrap();
    builder
        .finish(&mut backend, |runtime, backend| runtime.run(backend))
        .unwrap();

    assert_eq!("false", backend.line(0));
    assert_eq!("true", backend.line(1));
}
