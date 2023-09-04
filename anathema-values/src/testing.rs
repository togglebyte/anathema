use std::borrow::Cow;

use crate::{NodeId, Path, State, Value};

#[derive(Debug)]
pub struct TestState {
    name: Value<String>,
}

impl TestState {
    pub fn new() -> Self {
        Self {
            name: Value::new("Dirk Gently".to_string()),
        }
    }
}

impl State for TestState {
    fn get(&self, key: &Path, node_id: &NodeId) -> Option<Cow<'_, str>> {
        match key {
            Path::Key(s) => match s.as_str() {
                "name" => {
                    self.name.subscribe(node_id.clone());
                    Some((&self.name).into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn get_no_sub(&self, key: &Path) -> Option<Cow<'_, str>> {
        match key {
            Path::Key(s) => match s.as_str() {
                "name" => {
                    Some((&self.name).into())
                }
                _ => None,
            },
            _ => None,
        }
    }


    fn get_collection(&self, key: &Path) -> Option<crate::Collection> {
        todo!()
    }
}
