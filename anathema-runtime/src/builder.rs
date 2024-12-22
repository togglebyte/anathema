use anathema_default_widgets::register_default_widgets;
use anathema_templates::{Document, ToSourceKind};
use anathema_widgets::components::{Component, ComponentId, ComponentRegistry};
use anathema_widgets::{AttributeStorage, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, WidgetTree};

pub use crate::error::{Error, Result};
use crate::runtime::Runtime;

pub struct Builder {
    factory: Factory,
    document: Document,
    component_registry: ComponentRegistry,
}

impl Builder {
    /// Create a new builder
    pub fn new(document: Document) -> Self {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);
        Self {
            factory,
            document,
            component_registry: ComponentRegistry::new(),
        }
    }

    /// Registers a component as a template-only component.
    ///
    /// This component has no state or reacts to any events
    pub fn template<C: Component>(&mut self, ident: impl Into<String>, template: impl ToSourceKind) {
        _ = self.component(ident, template, (), ());
    }

    /// Registers a [Component] with the runtime.
    /// This returns a unique [ComponentId] that is used to send messages to the component.
    ///
    /// A component can only be used once in a template.
    /// If you want multiple instances, register the component as a prototype instead,
    /// see [RuntimeBuilder::prototype].
    pub fn component<C: Component>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        component: C,
        state: C::State,
    ) -> Result<ComponentId<C::Message>> {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.to_source_kind())?.into();
        self.component_registry.add_component(id, component, state);
        Ok(id.into())
    }

    /// Registers a [Component] as a prototype with the [Runtime],
    /// which allows for multiple instances of the component to exist the templates.
    pub fn prototype<FC, FS, C>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        proto: FC,
        state: FS,
    ) -> Result<()>
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> C::State,
        C: Component + 'static,
    {
        let ident = ident.into();
        let id = self.document.add_component(ident, template.to_source_kind())?.into();
        self.component_registry.add_prototype(id, proto, state);
        Ok(())
    }

    pub fn finish<B, F, U>(&mut self, backend: B, mut f: F) -> Result<()> 
        where F: FnMut(Runtime<'_, B>) -> Result<U>
    {
        let (blueprint, globals) = self.document.compile()?;
        let tree = WidgetTree::empty();
        let attribute_storage = AttributeStorage::empty();

        let inst = Runtime {
            backend,
            component_registry: &mut self.component_registry,
            components: Components::new(),
            document: &mut self.document,
            factory: &self.factory,
            tree,
            attribute_storage,
            floating_widgets: FloatingWidgets::empty(),
            changelist: ChangeList::empty(),
            dirty_widgets: DirtyWidgets::empty(),
            glyph_map: GlyphMap::empty(),
            blueprint: &blueprint,
            globals: &globals,
        };

        f(inst);

        Ok(())
    }
}
