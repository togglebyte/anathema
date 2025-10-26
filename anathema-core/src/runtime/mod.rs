//! Runtime execution system for Anathema templates.
//!
//! This module provides the runtime infrastructure for executing compiled templates,
//! managing components, and coordinating the widget system. It bridges the gap between
//! static template blueprints and the dynamic, interactive UI.
//!
//! # Overview
//!
//! The runtime system is responsible for:
//!
//! - **Component Management**: Instantiating and managing component lifecycle via [`Components`]
//! - **Widget Coordination**: Creating and organizing widgets through [`RegisteredWidgets`]
//! - **Expression Evaluation**: Evaluating template expressions to produce [`TemplateValue`]s
//! - **Element Tree**: Building and maintaining the element hierarchy
//!
//! # Components
//!
//! Components are reusable, stateful UI elements defined by the [`Component`] trait.
//! Each component has:
//!
//! - A `State` type that holds the component's data
//! - A `Message` type for handling user interactions
//! - A template describing its visual structure
//!
//! Components are registered with the runtime and can be instantiated multiple times,
//! each with its own state.
//!
//! ## Component Lifecycle
//!
//! 1. **Registration**: Components are registered either as single instances or as prototypes
//!    that can be instantiated multiple times.
//! 2. **Instantiation**: When a component is referenced in a template, the runtime creates
//!    an instance with its associated state.
//! 3. **State Updates**: Changes to component state trigger re-evaluation of dependent expressions
//!    and updates to the UI.
//! 4. **Message Handling**: User interactions generate messages that are routed to the appropriate
//!    component for processing.
//!
//! # Widgets
//!
//! Widgets are the renderable primitives of the UI. The [`Widget`] trait defines the interface
//! that all widgets must implement:
//!
//! ```rust,ignore
//! pub trait Widget: 'static {
//!     fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size;
//!     fn position(&mut self) -> Pos;
//!     fn paint(&mut self);
//!     fn describe(&self) -> &str;
//! }
//! ```
//!
//! Widgets are registered via [`RegisteredWidgets`] and instantiated by name from templates.
//!
//! ## Widget Registration
//!
//! ```rust,ignore
//! use anathema_core::runtime::RegisteredWidgets;
//!
//! let mut widgets = RegisteredWidgets::empty();
//! widgets.register_default::<MyWidget>("my_widget");
//! ```
//!
//! # Template Values
//!
//! [`TemplateValue`] represents the runtime values produced by evaluating template expressions.
//! It supports:
//!
//! - **Primitives**: Integers, floats, booleans, characters
//! - **Strings**: Both borrowed and owned strings
//! - **Colors**: Hex colors and named colors
//! - **Collections**: Lists and maps
//! - **Dynamic Values**: References to state values that can change over time
//! - **Ranges**: Numeric ranges for iteration
//!
//! Template values are the currency of the runtime - they flow from state through expressions
//! to widget attributes and rendering.
//!
//! # Expression Evaluation
//!
//! The runtime evaluates template expressions against the current state to produce values.
//! Expressions can:
//!
//! - Look up variables in the current scope
//! - Access component state
//! - Perform arithmetic and logical operations
//! - Call built-in functions
//! - Index into collections
//!
//! The evaluation system is implemented in the `eval` submodule.
//!
//! # Element Tree
//!
//! The runtime maintains a tree of elements that corresponds to the structure defined
//! by the template. Each element:
//!
//! - Has a unique [`ElementId`]
//! - May be associated with a widget
//! - Has attributes that configure its behavior
//! - Has zero or more child elements
//!
//! The element tree is built from the template blueprint and updated as state changes.
//!
//! # Children Iteration
//!
//! The [`Children`] iterator provides widgets with access to their child elements during
//! layout and rendering. It allows widgets to:
//!
//! - Iterate over direct children
//! - Query child count
//! - Access child widgets and their layout information
//!
//! # Module Organization
//!
//! - [`components`]: Component management and lifecycle
//! - `elements`: Element tree structure and management (internal)
//! - `eval`: Expression evaluation engine (internal)
//! - `widgets`: Widget trait and registration
//! - `functions`: Built-in template functions (internal)
//!
//! # Example: Widget with Children
//!
//! ```rust,ignore
//! use anathema_core::runtime::{Widget, Children};
//! use anathema_core::layout::Layout;
//! use anathema_geometry::Size;
//!
//! struct VStack;
//!
//! impl Widget for VStack {
//!     fn layout(&mut self, mut children: Children<'_, '_>, layout: &mut Layout) -> Size {
//!         let mut height = 0;
//!         let mut max_width = 0;
//!
//!         for child in children {
//!             let child_size = child.layout(layout);
//!             height += child_size.height;
//!             max_width = max_width.max(child_size.width);
//!         }
//!
//!         Size::new(max_width, height)
//!     }
//!
//!     fn position(&mut self) -> Pos {
//!         Pos::ZERO
//!     }
//!
//!     fn paint(&mut self) {
//!         // Paint implementation
//!     }
//! }
//! ```

pub use widgets::iter::Children;
pub use widgets::{RegisteredWidgets, Widget};

pub use crate::runtime::eval::values::TemplateValue;

pub mod components;
pub(crate) mod elements;
mod error;
pub(crate) mod eval;
pub(crate) mod functions;
pub(crate) mod widgets;
