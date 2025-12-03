use std::collections::HashMap;

use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Size};
use anathema_store::slab::{Generational, Key, SecondaryMap};

pub use self::iter::Children;
pub use self::layout::Layout;
use crate::attributes::{Attributes, WidgetAttributes};
use crate::elements::ElementId;

type WidgetFactory = Box<dyn for<'a, 'bp> Fn(&Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp>>;

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
    pub fn register_default<T>(&mut self, ident: impl Into<Box<str>>)
    where
        for<'bp> T: Widget<'bp> + Default,
    {
        self.registry.insert(
            ident.into(),
            Box::new(|_attr| {
                let inst = T::default();
                Box::new(inst)
            }),
        );
    }

    /// Register a widget type as longas it implements default
    pub fn register<F>(&mut self, ident: impl Into<Box<str>>, f: F)
    where
        for<'bp> F: 'bp + Fn(&Attributes<'bp>) -> Box<dyn Widget<'bp> + 'bp>,
    {
        self.registry
            .insert(ident.into(), Box::new(move |attr: &Attributes<'_>| f(attr)));
    }

    /// Create a widget from attributes
    pub fn make<'bp>(&self, ident: &str, attribs: &Attributes<'bp>) -> Result<Box<dyn Widget<'bp> + 'bp>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attribs);
        Ok(element)
    }
}

/// A widget.
///
/// Attributes should not be preserved on the widgets themselves
/// except to act as a cache between layout, position and paint.
/// Since an attribute can change as a result of a component event,
/// and this will not update any cached values.
pub trait Widget<'bp>: 'bp {
    /// Layout the widget
    fn layout(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        layout: &mut Layout,
    ) -> Size;

    /// Position the widget
    fn position(&mut self, children: Children<'_, 'bp>, attributes: WidgetAttributes<'_, 'bp>, pos: Pos);

    /// Paint the widget
    fn paint(
        &mut self,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
    );

    /// A function that described a widget in a debug context.
    fn describe(&self) -> &str {
        "<dyn Element>"
    }
}

impl std::fmt::Debug for dyn Widget<'_> {
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
