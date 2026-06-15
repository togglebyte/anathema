use std::collections::HashMap;

use crate::Attributes;
use crate::widgets::Widget;

type WidgetFactory = Box<dyn for<'a, 'bp> Fn(&Attributes<'bp>) -> Box<dyn Widget>>;

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
        T: Widget + Default,
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
        for<'bp> F: 'bp + Fn(&Attributes<'bp>) -> Box<dyn Widget>,
    {
        self.registry
            .insert(ident.into(), Box::new(move |attr: &Attributes<'_>| f(attr)));
    }

    /// Create a widget from attributes
    pub fn make<'bp>(&self, ident: &str, attribs: &Attributes<'bp>) -> Result<Box<dyn Widget>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attribs);
        Ok(element)
    }
}
