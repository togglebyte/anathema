use anathema_state::{CommonVal, Path, PendingValue, State, Subscriber, Value, ValueRef};
use run::TestCase;
mod run;

struct TestState {
    is_true: Value<bool>,
}

impl State for TestState {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }

    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        match path {
            Path::Key("is_true") => Some(self.is_true.value_ref(sub)),
            _ => None,
        }
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        match path {
            Path::Key("is_true") => Some(self.is_true.to_pending()),
            _ => None,
        }
    }
}

#[test]
fn basic() {
    let state = TestState { is_true: true.into() };
    TestCase::setup("test is_true")
        .build(state)
        .expect_frame("test Bool(true)")
        .with_state(0, |state| *state.is_true.to_mut() = false)
        .expect_frame("test Bool(false)");
}

#[test]
fn if_else() {
    let state = TestState { is_true: false.into() };
    TestCase::setup(
        r#"
if does.not.exist
    test "a"
else if !is_true
    test "b"
else
    test "c"
    "#,
    )
    .build(state)
    .expect_frame(
        r#"
<control flow>
    <if cond = false>
        test Str("a")
    <else cond = true>
        test Str("b")
    <else>
        test Str("c")
        "#,
    );
}
