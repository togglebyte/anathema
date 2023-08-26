use std::rc::Rc;

use anathema_values::Path;
use anathema_values::hashmap::HashMap;

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
