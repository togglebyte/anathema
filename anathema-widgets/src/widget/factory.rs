use std::collections::HashMap;
use std::marker::PhantomData;

use super::{AnyWidget, Widget};
use crate::Attributes;

pub struct DefaultFactory<W: AnyWidget + Default>(PhantomData<W>);

impl<W: 'static + AnyWidget + Default> WidgetFactory for DefaultFactory<W> {
    fn make(&self, _attribs: &Attributes<'_>) -> Box<dyn AnyWidget> {
        Box::<W>::default()
    }
}

pub struct Factory(HashMap<Box<str>, Box<dyn WidgetFactory>>);

impl Factory {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub(crate) fn make(&self, ident: &str, attribs: &Attributes<'_>) -> Box<dyn AnyWidget> {
        self.0.get(ident).unwrap().make(attribs)
    }

    pub fn register_widget(&mut self, ident: &str, factory: impl WidgetFactory + 'static) {
        self.0.insert(ident.into(), Box::new(factory));
    }

    pub fn register_default<W: 'static + Widget + Default>(&mut self, ident: &str) {
        let factory = DefaultFactory::<W>(PhantomData);
        self.0.insert(ident.into(), Box::new(factory));
    }
}

pub trait WidgetFactory {
    fn make(&self, attribs: &Attributes<'_>) -> Box<dyn AnyWidget>;
}
