//! Widget system for rendering UI elements.
//!
//! This module provides the widget trait and registration system that forms the rendering
//! layer of Anathema. Widgets are the concrete, renderable primitives that correspond to
//! template elements like `text`, `border`, `vstack`, etc.
//!
//! # Overview
//!
//! The widget system provides:
//!
//! - **Widget Trait**: The [`Widget`] trait defines the interface all widgets must implement
//! - **Registration**: [`RegisteredWidgets`] manages the set of available widget types
//! - **Children Iteration**: The [`Children`] iterator provides access to child widgets
//! - **Factory Pattern**: Widgets are created via factory functions for flexibility
//!
//! # Widget Lifecycle
//!
//! 1. **Registration**: Widget types are registered with [`RegisteredWidgets`] by name
//! 2. **Instantiation**: When a template references a widget by name, the factory creates an instance
//! 3. **Layout**: The widget calculates its size based on constraints and children
//! 4. **Positioning**: The widget determines its position within the parent
//! 5. **Painting**: The widget renders itself to the terminal backend
//!
//! # The Widget Trait
//!
//! All widgets must implement the [`Widget`] trait, which defines four key methods:
//!
//! - [`Widget::layout`]: Calculate the widget's size based on available space and children
//! - [`Widget::position`]: Return the widget's position (typically set by the parent)
//! - [`Widget::paint`]: Render the widget to the screen
//! - [`Widget::describe`]: Provide a debug description of the widget
//!
//! # Example Widget Implementation
//!
//! ```rust,ignore
//! use anathema_core::runtime::{Widget, Children};
//! use anathema_core::layout::Layout;
//! use anathema_geometry::{Pos, Size};
//!
//! struct Text {
//!     content: String,
//!     pos: Pos,
//! }
//!
//! impl Widget for Text {
//!     fn layout(&mut self, _children: Children<'_, '_>, _layout: &mut Layout) -> Size {
//!         // Calculate size based on text content
//!         Size::new(self.content.len() as u16, 1)
//!     }
//!
//!     fn position(&mut self) -> Pos {
//!         self.pos
//!     }
//!
//!     fn paint(&mut self) {
//!         // Render the text to the screen
//!     }
//!
//!     fn describe(&self) -> &str {
//!         "Text"
//!     }
//! }
//! ```
//!
//! # Widget Registration
//!
//! Widgets are registered by name, allowing templates to reference them:
//!
//! ```rust,ignore
//! use anathema_core::runtime::RegisteredWidgets;
//!
//! let mut widgets = RegisteredWidgets::empty();
//!
//! // Register a widget with default construction
//! widgets.register_default::<Text>("text");
//!
//! // Later, create an instance from a template
//! let widget = widgets.make("text", &attributes)?;
//! ```
//!
//! # Children
//!
//! Widgets can have children, accessed via the [`Children`] iterator during layout.
//! This allows container widgets like `vstack` or `border` to position and size
//! their child widgets appropriately.
//!
//! # Widget Tree
//!
//! The runtime maintains a tree of widget instances that mirrors the element tree
//! from the template. The [`Widgets`] type manages this tree structure.

use std::collections::HashMap;

use anathema_geometry::{Pos, Size};
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::attributes::Attributes;
use crate::layout::Layout;
use crate::runtime::elements::ElementId;
use crate::runtime::widgets::iter::Children;

/// Type alias for widget factory functions.
///
/// Widget factories receive element attributes and create widget instances.
/// This allows widgets to be configured based on template attributes at
/// creation time.
type WidgetFactory = Box<dyn Fn(&Attributes<'_>) -> Box<dyn Widget>>;

pub mod iter;

/// Registry of all available widget types.
///
/// This type manages the set of widget types that can be instantiated from templates.
/// Widgets are registered by name and created via factory functions, allowing for
/// flexible widget construction and configuration.
///
/// # Registration
///
/// Widgets can be registered using [`register_default`](Self::register_default) for
/// widgets that implement `Default`, or with custom factory functions for more
/// complex initialization.
///
/// # Example
///
/// ```rust,ignore
/// use anathema_core::runtime::RegisteredWidgets;
///
/// let mut widgets = RegisteredWidgets::empty();
/// widgets.register_default::<Text>("text");
/// widgets.register_default::<Border>("border");
/// widgets.register_default::<VStack>("vstack");
///
/// // Later, widgets are instantiated by name
/// let text_widget = widgets.make("text", &attributes)?;
/// ```
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
    /// Create a new empty widget registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::runtime::RegisteredWidgets;
    ///
    /// let widgets = RegisteredWidgets::empty();
    /// ```
    pub fn empty() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Register a widget type that implements `Default`.
    ///
    /// This is a convenience method for widgets that can be constructed with their
    /// default implementation. The widget factory ignores attributes and creates
    /// instances using `Default::default()`.
    ///
    /// # Parameters
    ///
    /// - `ident`: The name by which the widget will be referenced in templates
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::runtime::{Widget, RegisteredWidgets};
    ///
    /// #[derive(Default)]
    /// struct Text;
    ///
    /// impl Widget for Text {
    ///     // ... implementation
    /// }
    ///
    /// let mut widgets = RegisteredWidgets::empty();
    /// widgets.register_default::<Text>("text");
    /// ```
    pub fn register_default<T: Widget + Default>(&mut self, ident: impl Into<Box<str>>) {
        self.registry
            .insert(ident.into(), Box::new(|_attr| Box::<T>::default()));
    }

    /// Create a widget instance by name.
    ///
    /// Looks up the widget factory by name and invokes it with the provided attributes
    /// to create a widget instance.
    ///
    /// # Parameters
    ///
    /// - `ident`: The name of the widget to create
    /// - `attributes`: Element attributes to pass to the widget factory
    ///
    /// # Returns
    ///
    /// `Ok(Box<dyn Widget>)` if the widget exists, `Err(())` if not found.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::attributes::Attributes;
    ///
    /// let mut attributes = Attributes::empty();
    /// attributes.set("foreground", "blue");
    ///
    /// let widget = widgets.make("text", &attributes)?;
    /// ```
    pub fn make(&self, ident: &str, attributes: &Attributes<'_>) -> Result<Box<dyn Widget>, ()> {
        let Some(factory) = self.registry.get(ident) else { return Err(()) };
        let element = factory(attributes);
        Ok(element)
    }
}

/// The core widget trait for renderable UI elements.
///
/// All widgets in Anathema must implement this trait, which defines the interface
/// for layout calculation, positioning, and rendering. Widgets are the concrete
/// implementations of template elements like `text`, `border`, `vstack`, etc.
///
/// # Widget Responsibilities
///
/// Widgets are responsible for:
///
/// - **Layout**: Calculating their own size based on available space and children
/// - **Positioning**: Maintaining their position in the parent's coordinate space
/// - **Rendering**: Drawing themselves to the terminal backend
/// - **Description**: Providing a debug description for introspection
///
/// # Layout Process
///
/// The layout process is hierarchical and proceeds top-down:
///
/// 1. Parent widgets call [`layout`](Self::layout) on themselves
/// 2. Within [`layout`](Self::layout), they iterate over their [`Children`] and
///    calculate child layouts
/// 3. Each widget returns its calculated [`Size`]
/// 4. Parent widgets use child sizes to determine their own size
///
/// # Example Implementation
///
/// ```rust,ignore
/// use anathema_core::runtime::{Widget, Children};
/// use anathema_core::layout::Layout;
/// use anathema_geometry::{Pos, Size};
///
/// struct VStack {
///     pos: Pos,
/// }
///
/// impl Widget for VStack {
///     fn layout(&mut self, mut children: Children<'_, '_>, layout: &mut Layout) -> Size {
///         let mut total_height = 0;
///         let mut max_width = 0;
///
///         for child in children {
///             let child_size = child.layout(layout);
///             total_height += child_size.height;
///             max_width = max_width.max(child_size.width);
///         }
///
///         Size::new(max_width, total_height)
///     }
///
///     fn position(&mut self) -> Pos {
///         self.pos
///     }
///
///     fn paint(&mut self) {
///         // Render the container (if needed)
///     }
///
///     fn describe(&self) -> &str {
///         "VStack"
///     }
/// }
/// ```
pub trait Widget: 'static {
    /// Calculate the widget's layout and return its size.
    ///
    /// This method is called during the layout phase to determine how much space
    /// the widget needs. Container widgets should iterate over their children,
    /// calculate child layouts, and determine their own size based on the children.
    ///
    /// # Parameters
    ///
    /// - `children`: Iterator over child widgets for layout calculation
    /// - `layout`: Mutable reference to the layout system for updating child positions
    ///
    /// # Returns
    ///
    /// The calculated [`Size`] of this widget.
    ///
    /// # Notes
    ///
    /// - Leaf widgets (without children) can ignore the `children` parameter
    /// - Container widgets should iterate over children and call their layout methods
    /// - This method may be called multiple times as layout constraints change
    fn layout(&mut self, children: Children<'_, '_>, layout: &mut Layout) -> Size;

    /// Get the widget's current position.
    ///
    /// Returns the position of the widget in its parent's coordinate space.
    /// The position is typically set by the parent widget during layout.
    ///
    /// # Returns
    ///
    /// The widget's current [`Pos`]ition.
    fn position(&mut self) -> Pos;

    /// Render the widget to the screen.
    ///
    /// This method is called during the paint phase to render the widget's
    /// visual representation to the terminal backend. The widget should draw
    /// itself at its current position with its current size.
    ///
    /// # Notes
    ///
    /// - This method is called after layout is complete
    /// - Widgets should use their cached position and size for rendering
    /// - Paint order follows the element tree structure (depth-first)
    fn paint(&mut self);

    /// Provide a debug description of the widget.
    ///
    /// Returns a string describing the widget type, used for debugging and
    /// introspection. The default implementation returns a generic description.
    ///
    /// # Returns
    ///
    /// A static string describing this widget type.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn describe(&self) -> &str {
    ///     "Text"
    /// }
    /// ```
    fn describe(&self) -> &str {
        "<dyn Element>"
    }
}

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.describe())
    }
}

/// A node in the widget tree.
///
/// Each node represents a widget instance and tracks its children in the tree.
/// Nodes are indexed by [`ElementId`] and form a hierarchical structure that
/// mirrors the template element tree.
pub struct Node {
    // TODO: do we need the id here?
    /// The element ID for this node
    id: ElementId,
    /// The element IDs of this node's children
    children: Vec<ElementId>,
}

impl Node {
    /// Create a new widget tree node.
    ///
    /// # Parameters
    ///
    /// - `id`: The element ID for this node
    /// - `children`: The element IDs of the node's children
    ///
    /// # Returns
    ///
    /// A new `Node` instance.
    pub fn new(id: ElementId, children: Vec<ElementId>) -> Self {
        Self { id, children }
    }
}

/// The widget tree structure.
///
/// This type manages the hierarchical tree of widget instances that corresponds
/// to the element tree from the template. It provides the structural organization
/// for widget layout and rendering.
///
/// # Structure
///
/// The widget tree is stored as a flat map indexed by [`ElementId`], with each
/// node tracking its children. This allows efficient lookup while maintaining
/// the hierarchical relationships.
pub struct Widgets {
    /// Map from element IDs to widget tree nodes
    pub widgets: SecondaryMap<ElementId, Node>,
}

impl Widgets {
    /// Create a new empty widget tree.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use anathema_core::runtime::widgets::Widgets;
    ///
    /// let widgets = Widgets::empty();
    /// ```
    pub(crate) fn empty() -> Self {
        Self {
            widgets: SecondaryMap::empty(),
        }
    }
}
