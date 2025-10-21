use std::collections::HashMap;

use anathema_geometry::{Pos, Size};
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::attributes::Attributes;
use crate::layout::Layout;
use crate::runtime::widgets::iter::Children;
use crate::runtime::elements::ElementId;

type WidgetFactory = Box<dyn Fn(&Attributes<'_>) -> Box<dyn Widget>>;

pub mod iter;

/// All registered element types
pub struct RegisteredWidgets {
    registry: HashMap<Box<str>, WidgetFactory>,
}

impl RegisteredWidgets {
    pub fn empty() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn register_default<T: Widget + Default>(&mut self, ident: impl Into<Box<str>>) {
        self.registry
            .insert(ident.into(), Box::new(|_attr| Box::<T>::default()));
    }

    pub fn make(&self, ident: &str, attributes: &Attributes<'_>) -> Result<Box<dyn Widget>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attributes);
        Ok(element)
    }
}

/// An element ...
pub trait Widget: 'static {
    fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size;

    fn position(&mut self) -> Pos;

    fn paint(&mut self);

    fn describe(&self) -> &str {
        "<dyn Element>"
    }
}

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.describe())
    }
}

pub struct Node {
    // TODO: do we need the id here?
    id: ElementId,
    children: Vec<ElementId>,
}

impl Node {
    pub fn new(id: ElementId, children: Vec<ElementId>) -> Self {
        Self { id, children }
    }
}

/// The widget tree, constructed from the element tree
pub struct Widgets {
    pub widgets: SecondaryMap<ElementId, Node>,
}

impl Widgets {
    pub(crate) fn empty() -> Self {
        Self {
            widgets: SecondaryMap::empty(),
        }
    }

}
