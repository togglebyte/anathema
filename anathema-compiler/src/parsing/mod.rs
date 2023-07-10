use anathema_widget_core::{TextPath, Value};

mod attribute_parser;
pub(crate) mod parser;

#[derive(Debug, Default)]
pub struct Constants {
    strings: Vec<String>,
    texts: Vec<TextPath>,
    vaules: Vec<Value>,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            texts: vec![],
            vaules: vec![],
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
        match self.vaules.iter().position(|v| value.eq(v)) {
            Some(index) => index,
            None => {
                let index = self.vaules.len();
                self.vaules.push(value);
                index
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
        self.vaules.get(index)
    }
}
