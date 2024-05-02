use anathema_state::{List, Map, Value};
use run::TestCase;
mod run;

#[test]
fn future_dyn_if() {
    let state = Map::<bool>::empty();
    TestCase::setup(input!("future_dyn_if"))
        .build(state)
        .expect_frame(out!("future_dyn_if", 1))
        .with_state(0, |state| state.insert("value", true))
        .expect_frame(out!("future_dyn_if", 2));
}

#[test]
fn future_dyn_collection() {
    let state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    TestCase::setup(input!("future_dyn_collection"))
        .build(state)
        .expect_frame(out!("future_dyn_collection", 1))
        .with_state(0, move |state| state.insert("list", list))
        .expect_frame(out!("future_dyn_collection", 2));
}

#[test]
fn future_dyn_value() {
    let state = Map::empty();
    TestCase::setup(input!("future_dyn_value"))
        .build(state)
        .expect_frame(out!("future_dyn_value", 1))
        .with_state(0, |state| state.insert("value", "hello world"))
        .expect_frame(out!("future_dyn_value", 2));
}
