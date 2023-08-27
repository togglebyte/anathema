use std::rc::Rc;

use anathema_values::Path;
use anathema_values::hashmap::HashMap;

// A value is either a `Path` to a value in the the state, 
// a "static" value (set in the template and can't change during runtime),
// or a list of values.
#[derive(Debug)]
pub enum Value {
    Dyn(Path),
    Static(Rc<str>),
    List(Rc<[Value]>),
}

#[derive(Debug)]
pub struct Attributes(HashMap<String, Value>);

impl Attributes {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }
}
