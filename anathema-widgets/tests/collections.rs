use anathema_state::{List, Map, Value};
use run::TestCase;
mod run;

#[test]
fn collection_insert() {
    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);
    TestCase::setup(input!("collection_insert"))
        .build(state)
        .expect_frame(out!("collection_insert", 1))
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                list.push_back(100);
                list.insert(0, 99);
            }
        })
        .expect_frame(out!("collection_insert", 2));
}

#[test]
fn collection_remove() {
    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);

    TestCase::setup(input!("collection_remove"))
        .build(state)
        .expect_frame(out!("collection_remove", 1))
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                let _ = list.remove(0);
                let _ = list.remove(1);
            }
        })
        .expect_frame(out!("collection_remove", 2));
}

#[test]
fn collection_change() {
    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);
    TestCase::setup(input!("collection_change"))
        .build(state)
        .expect_frame(out!("collection_change", 1))
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                list.for_each(|val| *val *= 2);
            }
        })
        .expect_frame(out!("collection_change", 2));
}
