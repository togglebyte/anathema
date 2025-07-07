use std::collections::HashMap;

use anathema_value_resolver::Attributes;

use super::{AnyWidget, Widget};
use crate::error::{ErrorKind, Result};

pub struct Factory(HashMap<Box<str>, Box<dyn Fn(&Attributes<'_>) -> Box<dyn AnyWidget>>>);

impl Factory {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn make(&self, ident: &str, attribs: &Attributes<'_>) -> Result<Box<dyn AnyWidget>, ErrorKind> {
        let f = self.0.get(ident).ok_or(ErrorKind::InvalidElement(ident.to_string()))?;
        Ok((f)(attribs))
    }

    pub fn register_widget(&mut self, ident: &str, factory: impl Fn(&Attributes<'_>) -> Box<dyn AnyWidget> + 'static) {
        self.0.insert(ident.into(), Box::new(factory));
    }

    pub fn register_default<W: 'static + Widget + Default>(&mut self, ident: &str) {
        self.0.insert(ident.into(), Box::new(|_| Box::<W>::default()));
    }
}
