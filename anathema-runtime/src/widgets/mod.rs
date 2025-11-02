use std::collections::HashMap;

use anathema_geometry::{Pos, Size};
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

pub use self::iter::Children;
pub use self::layout::Layout;
use crate::attributes::Attributes;
use crate::elements::ElementId;

type WidgetFactory = Box<dyn Fn(&Attributes<'_>) -> Box<dyn Widget>>;

pub mod iter;
mod layout;

/// All registered widget types
#[derive(Default)]
pub struct RegisteredWidgets {
    registry: HashMap<Box<str>, WidgetFactory>,
}

impl std::fmt::Debug for RegisteredWidgets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.registry.keys()).finish()
    }
}

impl RegisteredWidgets {
    /// Create an empty set of registered widgets.
    pub fn empty() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a widget type as longas it implements default
    pub fn register_default<T: Widget + Default>(&mut self, ident: impl Into<Box<str>>) {
        self.registry
            .insert(ident.into(), Box::new(|_attr| Box::<T>::default()));
    }

    /// Register a widget type as longas it implements default
    pub fn register<F, T>(&mut self, ident: impl Into<Box<str>>, f: F)
    where
        T: Widget,
        F: Fn(&Attributes<'_>) -> T,
        F: 'static
    {
        self.registry.insert(ident.into(), Box::new(move |attr| Box::new(f(attr))));
    }

    /// Create a widget from attributes
    pub fn make(&self, ident: &str, attributes: &Attributes<'_>) -> Result<Box<dyn Widget>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attributes);
        Ok(element)
    }
}

/// A widget
pub trait Widget: 'static {
    /// Layout the widget
    fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size;

    /// Position the widget
    fn position(&mut self) -> Pos;

    /// Paint the widget
    fn paint(&mut self);

    /// A function that described a widget in a debug context.
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
    children: Vec<ElementId>,
}

impl Node {
    pub fn new(children: Vec<ElementId>) -> Self {
        Self { children }
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
