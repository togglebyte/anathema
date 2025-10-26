//! Core functionality for the Anathema TUI framework.
//!
//! `anathema-core` provides the fundamental building blocks for building terminal user interfaces
//! with Anathema. This crate contains the runtime system, template compilation, widget management,
//! and layout coordination that power Anathema applications.
//!
//! # Overview
//!
//! This crate is organized into several key modules:
//!
//! - **[`templates`]**: Template parsing, lexing, and compilation to blueprints
//! - **[`runtime`]**: Runtime execution of templates, component management, and widget system
//! - **[`attributes`]**: Element attribute storage and access
//! - **[`layout`]**: Layout calculation and region management for rendered elements
//! - **[`frontend`]**: Frontend abstractions (currently experimental)
//!
//! # Architecture
//!
//! The core workflow in Anathema follows this pattern:
//!
//! 1. **Template Compilation**: Template source code is parsed and compiled into [`Blueprint`]s
//!    via the [`Document`] API. Templates use a declarative syntax to describe UI structure.
//!
//! 2. **Runtime Execution**: The runtime system evaluates blueprints, instantiates widgets,
//!    manages component lifecycle, and handles state updates.
//!
//! 3. **Layout & Rendering**: Widgets calculate their layout based on constraints, and the
//!    layout system tracks the position and size of each element for rendering.
//!
//! # Template System
//!
//! Templates in Anathema are declarative descriptions of UI structure. They support:
//!
//! - **Widgets**: Named elements like `text`, `border`, `vstack`, etc.
//! - **Attributes**: Key-value pairs that configure widget behavior
//! - **Control Flow**: `if`, `for`, and `with` statements for dynamic content
//! - **Components**: Reusable UI elements with associated state and logic
//! - **Expressions**: Variables, operations, and function calls
//!
//! ## Example Template
//!
//! ```text
//! border [border_style: "rounded"]
//!     vstack
//!         text [foreground: "blue", bold: true] "Hello, World!"
//!
//!         for item in items
//!             text item.name
//! ```
//!
//! # Components
//!
//! Components are the primary way to build reusable, stateful UI elements. Each component
//! consists of:
//!
//! - A template describing its visual structure
//! - A state type implementing [`anathema_state::State`]
//! - Optional message handlers for user interaction
//!
//! Components are registered with the runtime and can be instantiated from templates
//! or programmatically.
//!
//! # Widget System
//!
//! Widgets are the renderable building blocks of the UI. The [`Widget`] trait defines
//! the interface for:
//!
//! - **Layout calculation**: Determining size based on constraints and children
//! - **Positioning**: Placement within the parent's coordinate space
//! - **Painting**: Rendering to the terminal backend
//!
//! Widgets are registered via [`RegisteredWidgets`] and instantiated by the runtime
//! when evaluating templates.
//!
//! # Attributes
//!
//! Element attributes are key-value pairs that configure widget behavior and appearance.
//! The [`Attributes`] type provides:
//!
//! - Type-safe attribute access via [`Attributes::get_as`]
//! - Dynamic attribute updates through [`RemoteCell`](anathema_store::remotecell::RemoteCell)
//! - Iteration over attribute keys and values
//!
//! ## Example
//!
//! ```rust
//! use anathema_core::attributes::Attributes;
//!
//! let mut attributes = Attributes::empty();
//! attributes.set("width", 100);
//! attributes.set("foreground", "red");
//!
//! // Type-safe access
//! let width: u32 = attributes.get_as("width").unwrap();
//! let color: &str = attributes.get_as("foreground").unwrap();
//! ```
//!
//! # Template Compilation
//!
//! The compilation process transforms template source into executable blueprints:
//!
//! ```rust
//! use anathema_core::templates::{Document, Variables};
//!
//! let mut doc = Document::new("text 'Hello, World!'");
//! let mut globals = Variables::new();
//!
//! let blueprint = doc.compile(&mut globals).expect("compilation failed");
//! ```
//!
//! The resulting [`Blueprint`] can then be executed by the runtime to create
//! the actual widget tree.
//!
//! # Expressions
//!
//! Templates support rich expression syntax for dynamic values:
//!
//! - **Literals**: `123`, `"hello"`, `true`, `'c'`, `#ff0000`
//! - **Variables**: `user.name`, `items[0]`
//! - **Arithmetic**: `count + 1`, `width * 2`
//! - **Comparison**: `score > 100`, `name == "Alice"`
//! - **Logic**: `enabled && visible`, `!disabled`
//! - **Ranges**: `0..10`
//! - **Collections**: `[1, 2, 3]`, `{name: "Alice", age: 30}`
//! - **Function calls**: `max(a, b)`, `len(items)`
//!
//! Expressions are compiled to the [`Expression`] enum and evaluated at runtime
//! to produce [`TemplateValue`]s.
//!
//! # State Management
//!
//! While state types are defined in the `anathema-state` crate, this crate provides
//! the runtime machinery for:
//!
//! - Binding component state to templates
//! - Tracking state changes and triggering updates
//! - Evaluating expressions against state values
//! - Managing the component lifecycle
//!
//! # Feature Flags
//!
//! This crate currently has no optional features. All functionality is enabled by default.
//!
//! # Stability
//!
//! This crate is in beta. While the core APIs are relatively stable, breaking changes
//! may occur between minor versions until 1.0 is released.
//!
//! [`Blueprint`]: templates::Blueprint
//! [`Document`]: templates::Document
//! [`Expression`]: templates::Expression
//! [`TemplateValue`]: runtime::TemplateValue
//! [`Widget`]: runtime::Widget
//! [`RegisteredWidgets`]: runtime::RegisteredWidgets
//! [`Attributes`]: attributes::Attributes

#[allow(unused_extern_crates)]
extern crate anathema_state as anathema;

pub mod attributes;
pub mod frontend;
pub mod layout;
pub mod runtime;
pub mod templates;

pub(crate) mod testing;
