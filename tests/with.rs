use anathema::component::*;
use anathema::prelude::*;

struct C;

impl Component for C {
    type State = String;
    type Message = ();
}

#[test]
fn eval_with() {
}
