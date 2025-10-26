//! Component trait definitions for Anathema.
//!
//! This module defines the core [`Component`] and [`AnyComponent`] traits that form
//! the foundation of Anathema's component system. Components are reusable, stateful
//! UI elements that encapsulate both presentation (via templates) and behavior
//! (via message handling).
//!
//! # Component System
//!
//! Components in Anathema provide a way to create modular, reusable UI elements with
//! their own state and logic. Each component is defined by implementing the [`Component`]
//! trait, which associates a state type and message type with the component.
//!
//! ## Component Architecture
//!
//! A component consists of:
//!
//! - **State**: Data that drives the component's appearance and behavior
//! - **Message**: Events that the component can handle (e.g., user input)
//! - **Template**: A declarative description of the component's UI structure
//!
//! ## The Component Trait
//!
//! The [`Component`] trait is intentionally minimal, serving primarily as a type-level
//! association between a component and its state and message types:
//!
//! ```rust
//! use anathema_core::runtime::components::Component;
//! use anathema_state::{State, CommonVal};
//!
//! struct Counter;
//!
//! #[derive(State)]
//! struct CounterState {
//!     count: CommonVal<i32>,
//! }
//!
//! enum CounterMessage {
//!     Increment,
//!     Decrement,
//! }
//!
//! impl Component for Counter {
//!     type State = CounterState;
//!     type Message = CounterMessage;
//! }
//! ```
//!
//! ## Type Erasure with AnyComponent
//!
//! The [`AnyComponent`] trait provides type erasure for components, allowing the runtime
//! to store and manipulate components of different concrete types in a uniform way.
//! This trait is automatically implemented for all types that implement [`Component`].
//!
//! The runtime uses `Box<dyn AnyComponent>` to store component instances, enabling
//! polymorphic component management without generic parameters.
//!
//! # State Management
//!
//! Component state must implement the `anathema_state::State` trait, which provides:
//!
//! - Change tracking for efficient UI updates
//! - Type-safe access to state fields
//! - Integration with the template expression system
//!
//! When component state changes, the runtime automatically re-evaluates dependent
//! template expressions and updates the affected portions of the UI.
//!
//! # Message Handling
//!
//! The `Message` associated type defines the events that a component can handle.
//! Messages typically represent user interactions like button clicks, text input,
//! or custom events.
//!
//! Message handling is implemented separately from the [`Component`] trait itself,
//! allowing for flexible event routing and processing.
//!
//! # Lifecycle
//!
//! Components have a managed lifecycle:
//!
//! 1. **Registration**: Components are registered with the runtime, either as single
//!    instances or as prototypes that can be instantiated multiple times.
//! 2. **Instantiation**: When referenced in a template, a component instance is created
//!    along with its initial state.
//! 3. **Active**: The component processes messages and state updates, triggering UI
//!    updates as needed.
//! 4. **Cleanup**: When no longer needed, component instances are dropped along with
//!    their state.
//!
//! # Debug Implementation
//!
//! Both [`Component`] and [`AnyComponent`] provide `Debug` implementations for
//! trait objects, enabling easier debugging and introspection of the component system.

/// The Component trait associates a component type with its state and message types.
///
/// This trait is the foundation of Anathema's component system. It defines the
/// relationship between a component implementation and its associated state and
/// message types.
///
/// # Type Parameters
///
/// - `State`: The state type that holds the component's data. Must implement
///   `anathema_state::State`.
/// - `Message`: The message type for events that the component can handle.
///
/// # Example
///
/// ```rust
/// use anathema_core::runtime::components::Component;
/// use anathema_state::{State, CommonVal};
///
/// struct TodoList;
///
/// #[derive(State)]
/// struct TodoListState {
///     items: CommonVal<Vec<String>>,
///     completed: CommonVal<Vec<bool>>,
/// }
///
/// enum TodoMessage {
///     Add(String),
///     Toggle(usize),
///     Remove(usize),
/// }
///
/// impl Component for TodoList {
///     type State = TodoListState;
///     type Message = TodoMessage;
/// }
/// ```
///
/// # Design Notes
///
/// The trait has no methods because it serves purely as a type-level association.
/// The actual behavior of components is implemented through separate message handlers
/// and the state's implementation of the `State` trait.
pub trait Component: 'static {
    /// The state type that holds this component's data.
    ///
    /// This type must implement `anathema_state::State`, which provides change
    /// tracking and integration with the template system.
    type State;

    /// The message type for events this component can handle.
    ///
    /// Messages represent user interactions, timer events, or other signals
    /// that the component needs to process. Common examples include button
    /// clicks, text input, or custom domain events.
    type Message;
}

/// Type-erased component trait for runtime polymorphism.
///
/// This trait provides type erasure for components, allowing the runtime to store
/// and manipulate components of different concrete types uniformly. It is automatically
/// implemented for all types that implement [`Component`].
///
/// # Type Erasure
///
/// The runtime stores components as `Box<dyn AnyComponent>`, enabling:
///
/// - Storage of heterogeneous component types in collections
/// - Dynamic component instantiation based on template references
/// - Polymorphic component management without generic parameters
///
/// # Automatic Implementation
///
/// You never need to implement this trait manually - it is automatically implemented
/// for any type that implements [`Component`]:
///
/// ```rust
/// use anathema_core::runtime::components::{Component, AnyComponent};
///
/// struct MyComponent;
///
/// impl Component for MyComponent {
///     type State = ();
///     type Message = ();
/// }
///
/// // AnyComponent is automatically implemented
/// let component: Box<dyn AnyComponent> = Box::new(MyComponent);
/// ```
///
/// # Debug Representation
///
/// The trait object `dyn AnyComponent` has a custom `Debug` implementation that
/// displays as `<component>`, useful for debugging and introspection.
pub trait AnyComponent: 'static {}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<component>")
    }
}

impl<T: Component> AnyComponent for T {}
