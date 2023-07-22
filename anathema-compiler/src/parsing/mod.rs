use anathema_values::{Path, TextPath, Value, PathId};

mod attribute_parser;
mod fields;
pub(crate) mod parser;

#[derive(Debug, Default)]
pub struct Constants {
    strings: Vec<String>,
    texts: Vec<TextPath>,
    values: Vec<Value>,
    paths: Vec<Path>,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            texts: vec![],
            values: vec![],
            paths: vec![],
        }
    }

    fn store_string(&mut self, key: impl Into<String>) -> usize {
        let key = key.into();
        match self.strings.iter().position(|i| key.eq(i)) {
            Some(index) => index,
            None => {
                let index = self.strings.len();
                self.strings.push(key);
                index
            }
        }
    }

    fn store_text(&mut self, text: TextPath) -> usize {
        match self.texts.iter().position(|t| text.eq(t)) {
            Some(index) => index,
            None => {
                let index = self.texts.len();
                self.texts.push(text);
                index
            }
        }
    }

    pub fn store_value(&mut self, value: Value) -> usize {
        match self.values.iter().position(|v| value.eq(v)) {
            Some(index) => index,
            None => {
                let index = self.values.len();
                self.values.push(value);
                index
            }
        }
    }

    pub fn store_path(&mut self, path: Path) -> PathId {
        match self.paths.iter().position(|p| path.eq(p)) {
            Some(index) => index.into(),
            None => {
                let index = self.paths.len();
                self.paths.push(path);
                index.into()
            }
        }
    }

    pub fn lookup_string(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    pub fn lookup_text(&self, index: usize) -> Option<&TextPath> {
        self.texts.get(index)
    }

    pub fn lookup_value(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn lookup_path(&self, path_id: PathId) -> Option<&Path> {
        self.paths.get(*path_id)
    }
}
