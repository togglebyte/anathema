use anathema_state::{List, Map, Value};
use run::TestCase;
mod run;

#[test]
fn future_dyn_if() {
    let template = r#"
if value
    test "hello"
        "#;

    let f1 = r#"
<control flow>
    <if cond = false>
        test Str("hello")
        "#;

    let f2 = r#"
<control flow>
    <if cond = true>
        test Str("hello")
        "#;

    let state = Map::<bool>::empty();
    TestCase::setup(template)
        .build(state)
        .expect_frame(f1)
        .with_state(0, |state| state.insert("value", true))
        .expect_frame(f2);
}

#[test]
fn future_dyn_collection() {
    let template = r#"
for value in list
    test value
    "#;

    let f1 = "<for>";

    let f2 = r#"
<for>
    <iter binding = value, index = 0>
        test Int(1)
    <iter binding = value, index = 1>
        test Int(2)
    <iter binding = value, index = 2>
        test Int(3)
    "#;

    let state = Map::<List<_>>::empty();
    let list = Value::<List<_>>::from_iter([1, 2, 3]);
    TestCase::setup(template)
        .build(state)
        .expect_frame(f1)
        .with_state(0, move |state| state.insert("list", list))
        .expect_frame(f2);
}

#[test]
fn future_dyn_value() {
    let f2 = r#"test Str("hello world")"#;

    let state = Map::empty();
    TestCase::setup("test value")
        .build(state)
        .expect_frame("test")
        .with_state(0, |state| state.insert("value", "hello world"))
        .expect_frame(f2);
}
