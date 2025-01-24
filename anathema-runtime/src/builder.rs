use std::sync::atomic::Ordering;
use std::time::Instant;

use anathema_backend::Backend;
use anathema_default_widgets::register_default_widgets;
use anathema_geometry::Size;
use anathema_state::{Changes, States};
use anathema_strings::HStrings;
use anathema_templates::{Document, ToSourceKind};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::{AssociatedEvents, Component, ComponentId, ComponentRegistry, Emitter, FocusQueue, ViewMessage};
use anathema_widgets::layout::Viewport;
use anathema_widgets::{ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, WidgetTree};
use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};

pub use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::REBUILD;

pub struct Builder {
    factory: Factory,
    document: Document,
    component_registry: ComponentRegistry,
    emitter: Emitter,
    message_receiver: flume::Receiver<ViewMessage>,
}

impl Builder {
    /// Create a new builder
    pub fn new(document: Document) -> Self {
        let mut factory = Factory::new();
        register_default_widgets(&mut factory);

        let (tx, message_receiver) = flume::unbounded();
        let emitter = tx.into();

        Self {
            factory,
            document,
            component_registry: ComponentRegistry::new(),
            emitter,
            message_receiver,
        }
    }
    
    /// Returns an [Emitter] to send messages to components
    pub fn emitter(&self) -> Emitter {
        self.emitter.clone()
    }

    /// Registers a component as a template-only component.
    ///
    /// This component has no state or reacts to any events
    pub fn template(&mut self, ident: impl Into<String>, template: impl ToSourceKind) {
        _ = self.prototype(ident, template, || (), || ());
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

    /// Registers a [Component] with the runtime as long as the component and the associated state
    /// implements the `Default` trait.
    /// This returns a unique [ComponentId] that is used to send messages to the component.
    pub fn from_default<C>(
        &mut self,
        ident: impl Into<String>,
        template: impl ToSourceKind,
    ) -> Result<ComponentId<C::Message>>
    where
        C: Component + Default,
        C::State: Default,
    {
        let component = C::default();
        let state = C::State::default();
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

    pub fn finish<F, U>(mut self, size: Size, mut f: F) -> Result<U>
    where
        F: FnMut(&mut Runtime) -> Result<U>,
    {
        #[cfg(feature = "profile")]
        let _puffin_server = {
            let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
            let server = puffin_http::Server::new(&server_addr).unwrap();
            puffin::set_scopes_on(true);
            server
        };

        let (blueprint, globals) = self.document.compile()?;
        let viewport = Viewport::new(size);

        let fps = 30;
        let sleep_micros: u128 = ((1.0 / fps as f64) * 1000.0 * 1000.0) as u128;

        let watcher = self.set_watcher()?;

        let mut inst = Runtime {
            component_registry: self.component_registry,
            components: Components::new(),
            document: self.document,
            factory: self.factory,
            states: States::new(),
            floating_widgets: FloatingWidgets::empty(),
            changelist: ChangeList::empty(),
            dirty_widgets: DirtyWidgets::empty(),
            assoc_events: AssociatedEvents::new(),
            focus_queue: FocusQueue::new(),
            glyph_map: GlyphMap::empty(),
            blueprint,
            globals,
            changes: Changes::empty(),
            viewport,
            message_receiver: self.message_receiver,
            emitter: self.emitter,
            fps,
            sleep_micros,
            dt: Instant::now(),
            _watcher: Some(watcher),
        };

        loop {
            f(&mut inst)?;
            inst.reload();
        }
    }

    fn set_watcher(&mut self) -> Result<RecommendedWatcher> {
        let paths = self
            .document
            .template_paths()
            .filter_map(|p| p.canonicalize().ok())
            .collect::<Vec<_>>();

        let mut watcher = recommended_watcher(move |event: std::result::Result<Event, _>| match event {
            Ok(event) => match event.kind {
                notify::EventKind::Create(_) | notify::EventKind::Remove(_) | notify::EventKind::Modify(_) => {
                    if paths.iter().any(|p| event.paths.contains(p)) {
                        REBUILD.store(true, Ordering::Relaxed);
                    }
                }
                notify::EventKind::Any | notify::EventKind::Access(_) | notify::EventKind::Other => (),
            },
            Err(_err) => (),
        })?;

        for path in self.document.template_paths() {
            let path = path.canonicalize().unwrap();

            if let Some(parent) = path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }

        Ok(watcher)
    }
}
