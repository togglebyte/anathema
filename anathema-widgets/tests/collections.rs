use anathema_state::{List, Map, Value};
use run::TestCase;
mod run;

#[test]
fn collection_insert() {
    let template = r#"
        for val in list
            test val
    "#;

    let f1 = r#"
<for>
    <iter binding = val, index = 0>
        test Int(1)
    <iter binding = val, index = 1>
        test Int(2)
    <iter binding = val, index = 2>
        test Int(3)
        "#;

    let f2 = r#"
<for>
    <iter binding = val, index = 0>
        test Int(99)
    <iter binding = val, index = 1>
        test Int(1)
    <iter binding = val, index = 2>
        test Int(2)
    <iter binding = val, index = 3>
        test Int(3)
    <iter binding = val, index = 4>
        test Int(100)
            "#;

    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);
    TestCase::setup(template)
        .build(state)
        .expect_frame(f1)
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                list.push_back(100);
                list.insert(0, 99);
            }
        })
        .expect_frame(f2);
}

#[test]
fn collection_remove() {
    let template = r#"
for val in list
    test val
        "#;

    let f1 = r#"
<for>
    <iter binding = val, index = 0>
        test Int(1)
    <iter binding = val, index = 1>
        test Int(2)
    <iter binding = val, index = 2>
        test Int(3)
        "#;

    let f2 = r#"
<for>
    <iter binding = val, index = 1>
        test Int(2)
        "#;

    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);

    TestCase::setup(template)
        .build(state)
        .expect_frame(f1)
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                let _ = list.remove(0);
                let _ = list.remove(1);
            }
        })
        .expect_frame(f2);
}

#[test]
fn collection_change() {
    let template = r#"
for val in list
    test val
    "#;

    let f1 = r#"
<for>
    <iter binding = val, index = 0>
        test Int(1)
    <iter binding = val, index = 1>
        test Int(2)
    <iter binding = val, index = 2>
        test Int(3)
    "#;

    let f2 = r#"
<for>
    <iter binding = val, index = 0>
        test Int(2)
    <iter binding = val, index = 1>
        test Int(4)
    <iter binding = val, index = 2>
        test Int(6)
    "#;

    let mut state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    state.insert("list", list);
    TestCase::setup(template)
        .build(state)
        .expect_frame(f1)
        .with_state(0, |state| {
            if let Some(list) = state.to_mut().get_mut("list") {
                list.for_each(|val| *val *= 2);
            }
        })
        .expect_frame(f2);
}
