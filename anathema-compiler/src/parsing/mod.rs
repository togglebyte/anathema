use anathema_widget_core::{TextPath, Value};

mod attribute_parser;
pub(crate) mod parser;

#[derive(Debug, Default)]
pub struct Constants {
    idents: Vec<String>,
    texts: Vec<TextPath>,
    attribs: Vec<Value>,
}

impl Constants {
    pub fn new() -> Self {
        Self {
            // TODO: idents are also used for string interning, so "idents" might not be
            //       the best name. Should this simply be called "strings"?
            idents: vec![],
            texts: vec![],
            attribs: vec![],
        }
    }

    // Store an ident in consts.
    // This will not store duplicates
    fn store_ident(&mut self, key: impl Into<String>) -> usize {
        let key = key.into();
        match self.idents.iter().position(|i| key.eq(i)) {
            Some(index) => index,
            None => {
                let index = self.idents.len();
                self.idents.push(key);
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

    pub fn store_attribute(&mut self, value: Value) -> usize {
        match self.attribs.iter().position(|v| value.eq(v)) {
            Some(index) => index,
            None => {
                let index = self.attribs.len();
                self.attribs.push(value);
                index
            }
        }
    }

    pub fn lookup_ident(&self, index: usize) -> Option<&str> {
        self.idents.get(index).map(String::as_str)
    }

    pub fn lookup_text(&self, index: usize) -> Option<&TextPath> {
        self.texts.get(index)
    }

    // TODO: calling this attribute is a bit misleading.
    // an attribute on a node: `node [attribute: here]`, but this 
    // does more than load attributes, it loads conditions for `if` and 
    // data bindings for `for`
    pub fn lookup_attrib(&self, index: usize) -> Option<&Value> {
        self.attribs.get(index)
    }
}
