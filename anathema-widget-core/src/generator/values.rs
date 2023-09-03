use std::rc::Rc;

use anathema_values::{Path, ScopeValue};
use anathema_values::hashmap::HashMap;

#[derive(Debug)]
pub struct Attributes(HashMap<String, ScopeValue>);

impl Attributes {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn set(&mut self, key: impl Into<String>, value: ScopeValue) {
        self.0.insert(key.into(), value);
    }
}
