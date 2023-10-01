use anathema_values::testing::*;
use anathema_values::*;

fn main() {
    let state = TestState::new();
    let path = Path::from("generic_map");
    let path = path.compose("inner");
    let path = path.compose("second");
    let x = state.get(&path, None);
    panic!("{x:#?}");
}
