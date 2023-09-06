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
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<Cow<'_, str>> {
        match key {
            Path::Key(s) => match s.as_str() {
                "name" => {
                    if let Some(node_id) = node_id.cloned() {
                        self.name.subscribe(node_id);
                    }
                    Some((&self.name).into())
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn get_collection(&self, key: &Path, node_id: Option<&NodeId>) -> Option<crate::Collection> {
        todo!()
    }
}
