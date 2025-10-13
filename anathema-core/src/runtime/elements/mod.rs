use std::collections::HashMap;

use anathema_geometry::{Pos, Size};

pub use self::tree::Children;
pub(crate) use self::tree::{ElementId, InsertNode, Elements};
use crate::attributes::Attributes;
use crate::layout::Layout;

mod eval;
mod tree;

type ElementFactory = Box<dyn Fn(&Attributes) -> Box<dyn Element>>;

/// All registered element types
pub struct RegisteredElements {
    registry: HashMap<Box<str>, ElementFactory>,
}

impl RegisteredElements {
    pub fn empty() -> Self {
        Self {
            registry: HashMap::new(),        
        }
    }

    pub fn register_default<T: Element + Default>(&mut self, ident: impl Into<Box<str>>) {
        self.registry
            .insert(ident.into(), Box::new(|_attr| Box::<T>::default()));
    }

    pub fn make(&self, ident: &str, attributes: &Attributes) -> Result<Box<dyn Element>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attributes);
        Ok(element)
    }
}

/// An element ...
pub trait Element: 'static {
    fn layout(&mut self, children: Children<'_>, layout: &mut Layout) -> Size;

    fn position(&mut self) -> Pos;

    fn paint(&mut self);

    fn describe(&self) -> &str {
        "<dyn Element>"
    }
}

impl std::fmt::Debug for dyn Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.describe())
    }
}
