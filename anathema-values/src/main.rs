use anathema_values::testing::*;
use anathema_values::*;

fn main() {
    let state = TestState::new();
    let path = Path::from("generic_list");
    let path = path.compose(0);
    let path = path.compose(0);
    let x = state.get(&path, None);
    panic!("{x:#?}");
}
