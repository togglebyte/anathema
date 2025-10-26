//! Component management and lifecycle.
//!
//! This module provides the infrastructure for managing components in the Anathema runtime.
//! It handles component registration, instantiation, and state management, supporting both
//! single-instance components and prototype-based component creation.
//!
//! # Overview
//!
//! The component system allows for:
//!
//! - **Registration**: Components can be registered as single instances or as prototypes
//! - **Instantiation**: Creating component instances on demand from templates
//! - **State Management**: Tracking and accessing component state
//! - **Lifecycle**: Managing component creation and cleanup
//!
//! # Components
//!
//! A component in Anathema consists of:
//!
//! - An implementation of the [`Component`] trait
//! - Associated state implementing `anathema_state::State`
//! - A template blueprint defining its visual structure
//!
//! Components are identified by [`ComponentId`] at runtime and [`ComponentBlueprintId`]
//! in the template system.
//!
//! # Component Types
//!
//! There are two types of component registrations:
//!
//! ## Single Instance Components
//!
//! Single instance components are registered once and used directly. Each reference to
//! the component in a template uses the same instance and state.
//!
//! ```rust,ignore
//! components.insert_component(
//!     blueprint_id,
//!     MyComponent,
//!     MyComponentState::default(),
//! );
//! ```
//!
//! ## Prototype Components
//!
//! Prototype components are registered with factory functions that create new instances
//! on demand. Each reference to the component in a template creates a new instance with
//! fresh state.
//!
//! ```rust,ignore
//! components.insert_prototype(
//!     blueprint_id,
//!     Box::new(|| Box::new(MyComponent)),
//!     Box::new(|| Box::new(MyComponentState::default())),
//! );
//! ```
//!
//! # Component Storage
//!
//! The [`Components`] struct manages all component instances using:
//!
//! - [`GenSlab`]: Generational slab allocator for component instances with stable IDs
//! - [`SecondaryMap`]: Maps blueprint IDs to component lookup information
//!
//! This design allows efficient component lookup by ID while supporting both direct
//! instances and factory-based instantiation.
//!
//! # State Access
//!
//! Component state can be accessed through the [`Components`] API:
//!
//! ```rust,ignore
//! // Immutable access
//! if let Some(state) = components.get_state(component_id) {
//!     // Access state through Value wrapper
//! }
//!
//! // Mutable access
//! if let Some(state) = components.get_state_mut(component_id) {
//!     // Modify state, triggering updates
//! }
//! ```
//!
//! State is wrapped in a `Value` type from `anathema-state`, which provides change
//! tracking and notification.
//!
//! # Component Lifecycle
//!
//! 1. **Registration**: Component is registered with the runtime
//! 2. **Blueprint Mapping**: Component blueprint ID is mapped to the component
//! 3. **Instantiation**: When referenced in a template, an instance is created
//! 4. **Active**: Component processes state updates and messages
//! 5. **Cleanup**: When no longer referenced, component is dropped
//!
//! # Example
//!
//! ```rust,ignore
//! use anathema_core::runtime::components::{Component, Components};
//! use anathema_state::{State, CommonVal};
//!
//! #[derive(State)]
//! struct CounterState {
//!     count: CommonVal<i32>,
//! }
//!
//! struct Counter;
//!
//! impl Component for Counter {
//!     type State = CounterState;
//!     type Message = ();
//! }
//!
//! let mut components = Components::empty();
//! let component_id = components.insert_component(
//!     blueprint_id,
//!     Counter,
//!     CounterState { count: CommonVal::new(0) },
//! );
//!
//! // Access component state
//! if let Some(state) = components.get_state(component_id) {
//!     println!("Count: {}", state.to_ref().count);
//! }
//! ```

use anathema::Value;
use anathema_state::State;
use anathema_store::key;
use anathema_store::slab::{GenSlab, SecondaryMap};

pub use self::component::{AnyComponent, Component};
use crate::templates::ComponentBlueprintId;

key!(ComponentId, Debug, Copy, Clone);

/// Type alias for component factory functions.
///
/// This function type creates new instances of type-erased components.
/// It's used for prototype-based component registration where each
/// template reference creates a fresh component instance.
pub(crate) type FnComp = Box<dyn Fn() -> Box<dyn AnyComponent>>;

/// Type alias for state factory functions.
///
/// This function type creates new instances of type-erased state.
/// It's used for prototype-based component registration where each
/// component instance gets its own fresh state.
pub(crate) type FnState = Box<dyn Fn() -> Box<dyn State>>;

mod component;

/// An entry in the component registry.
///
/// Each entry stores a component instance along with its state and metadata
/// about how it was created.
struct Entry {
    /// The type-erased component instance
    component: Box<dyn AnyComponent>,
    /// The component's state, wrapped in a Value for change tracking
    state: Value<Box<dyn State>>,
    /// How this component was created (direct or from prototype)
    kind: ComponentKind,
}

/// Distinguishes between directly registered components and prototype instances.
enum ComponentKind {
    /// A component that was directly registered as a single instance
    Component,
    /// A component instance created from a prototype factory
    PrototypeInstance,
}

/// Blueprint lookup strategy for resolving component references.
///
/// When a template references a component by blueprint ID, this enum
/// determines how to obtain the component instance.
enum Lookup {
    /// Factory functions that create new instances on each lookup
    Prototype(FnComp, FnState),
    /// Direct reference to an existing component instance
    Component(ComponentId),
}

/// Component registry and manager.
///
/// This type manages all components in the runtime, handling both single-instance
/// components and prototype-based component creation. It maintains the mapping
/// between blueprint IDs (from templates) and actual component instances.
///
/// # Storage
///
/// - `instances`: Stores all component instances with stable IDs
/// - `blueprints`: Maps template blueprint IDs to component lookup strategies
///
/// # Thread Safety
///
/// This type is not thread-safe and is intended to be used on a single thread
/// within the runtime.
pub struct Components {
    /// Storage for all component instances
    instances: GenSlab<ComponentId, Entry>,
    /// Mapping from blueprint IDs to component resolution strategies
    blueprints: SecondaryMap<ComponentBlueprintId, Lookup>,
}

impl Components {
    /// Create a new empty component registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::runtime::components::Components;
    ///
    /// let components = Components::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            instances: GenSlab::empty(),
            blueprints: SecondaryMap::empty(),
        }
    }

    /// Register a single-instance component.
    ///
    /// This creates a single component instance that will be shared across all
    /// template references to this blueprint ID. The component and its state are
    /// created once and reused.
    ///
    /// # Parameters
    ///
    /// - `blueprint_id`: The blueprint ID from the template system
    /// - `component`: The component instance
    /// - `state`: The initial state for the component
    ///
    /// # Returns
    ///
    /// A [`ComponentId`] that uniquely identifies this component instance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let component_id = components.insert_component(
    ///     blueprint_id,
    ///     MyComponent,
    ///     MyState::default(),
    /// );
    /// ```
    pub(crate) fn insert_component(
        &mut self,
        blueprint_id: ComponentBlueprintId,
        component: impl AnyComponent,
        state: impl State,
    ) -> ComponentId {
        let entry = Entry {
            component: Box::new(component),
            state: Value::new(Box::new(state)),
            kind: ComponentKind::Component,
        };
        let component_id = self.instances.insert(entry);
        self.blueprints.insert(blueprint_id, Lookup::Component(component_id));
        component_id
    }

    /// Register a prototype component with factory functions.
    ///
    /// This registers factory functions that will create new component instances
    /// on demand. Each template reference to this blueprint ID will get its own
    /// fresh instance with separate state.
    ///
    /// # Parameters
    ///
    /// - `blueprint_id`: The blueprint ID from the template system
    /// - `component`: Factory function that creates component instances
    /// - `state`: Factory function that creates state instances
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// components.insert_prototype(
    ///     blueprint_id,
    ///     Box::new(|| Box::new(MyComponent)),
    ///     Box::new(|| Box::new(MyState::default())),
    /// );
    /// ```
    pub(crate) fn insert_prototype(&mut self, blueprint_id: ComponentBlueprintId, component: FnComp, state: FnState) {
        self.blueprints
            .insert(blueprint_id, Lookup::Prototype(component, state));
    }

    /// Get or create a component instance by blueprint ID.
    ///
    /// If the blueprint references a single-instance component, returns its ID.
    /// If the blueprint references a prototype, creates a new instance and returns
    /// its ID.
    ///
    /// # Parameters
    ///
    /// - `id`: The blueprint ID from the template
    ///
    /// # Returns
    ///
    /// A [`ComponentId`] for the component instance.
    ///
    /// # Panics
    ///
    /// Currently panics (via `todo!()`) if the blueprint ID is not found. This will
    /// be improved in future versions to return a proper error.
    pub(crate) fn by_blueprint_id(&mut self, id: ComponentBlueprintId) -> ComponentId {
        match self.blueprints.get(id) {
            Some(Lookup::Component(id)) => *id,
            Some(Lookup::Prototype(comp, state)) => {
                let entry = Entry {
                    component: comp(),
                    state: Value::new(state()),
                    kind: ComponentKind::PrototypeInstance,
                };
                self.instances.insert(entry)
            }
            None => todo!(),
        }
    }

    /// Placeholder for future component lookup by ID.
    ///
    /// This method is currently unimplemented and reserved for future use.
    pub(crate) fn by_component_id(&self) {}

    /// Get immutable access to a component's state.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The ID of the component whose state to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&Value<Box<dyn State>>)` if the component exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(state) = components.get_state(component_id) {
    ///     // Access state through the Value wrapper
    ///     let state_ref = state.to_ref();
    /// }
    /// ```
    pub(crate) fn get_state(&self, component_id: ComponentId) -> Option<&Value<Box<dyn State>>> {
        let inst = self.instances.get(component_id)?;
        Some(&inst.state)
    }

    /// Get mutable access to a component's state.
    ///
    /// Mutations to the state will trigger change tracking and potentially cause
    /// UI updates.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The ID of the component whose state to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&mut Value<Box<dyn State>>)` if the component exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(state) = components.get_state_mut(component_id) {
    ///     // Modify state
    ///     let mut state_mut = state.to_mut();
    ///     // Changes will trigger updates
    /// }
    /// ```
    pub(crate) fn get_state_mut(&mut self, component_id: ComponentId) -> Option<&mut Value<Box<dyn State>>> {
        let inst = self.instances.get_mut(component_id)?;
        Some(&mut inst.state)
    }
}

impl Default for Components {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Debug for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<components>")
    }
}
